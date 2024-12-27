use anyhow::Result;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{OnceLock, mpsc},
    thread,
    time::{Duration, Instant},
};

use ratatui::crossterm::event::{self, Event as CTEvent, KeyModifiers};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::{app::AppContext, debug::log_to_file};

/// App events.
#[derive(Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,

    /// Key event.
    Key(KeyEvent),
}

/// An event handler.
pub struct EventHandler {
    /// Event sender channel.
    sx: mpsc::Sender<Event>,

    /// Event receiver channel.
    rx: mpsc::Receiver<Event>,

    /// Event handler thread.
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Creates a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);

        let (sx, rx) = mpsc::channel();
        let sx1 = sx.clone();

        let handler = thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                // Emit key events
                let timeout = tick_rate.saturating_sub(last_tick.elapsed());
                if event::poll(timeout).expect("unable to poll events") {
                    if let CTEvent::Key(e) = event::read().expect("unable to read event") {
                        let _ = sx1.send(Event::Key(e));
                    };
                }

                // Emit tick events
                if last_tick.elapsed() >= tick_rate {
                    let _ = sx1.send(Event::Tick);
                    last_tick = Instant::now();
                }
            }
        });

        Self { sx, rx, handler }
    }

    pub(super) fn next(&self) -> Result<Event> {
        Ok(self.rx.recv()?)
    }

    pub fn handle(&self, ctx: &mut AppContext) -> Result<()> {
        match self.next()? {
            Event::Tick => {
                // log_to_file("tick");
            }
            Event::Key(key) => {
                // Always quit on <ctrl-c>.
                if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                    ctx.should_quit = true;
                    return Ok(());
                }

                // Check for registered key events.
                if let Some(callback) = ctx.register.get_event(key) {
                    (callback)((ctx, key));
                }
                if let Some(callback) = ctx.register.get_event(RegisterEvent::All) {
                    (callback)((ctx, key));
                }
                // if let Some(callback) = &ctx.register.capture_all {
                //     let callback = Rc::clone(callback);
                //     (callback)(ctx);
                // }
            }
        }

        Ok(())
    }
}

type KeyEventCallback = dyn Fn((&mut AppContext, KeyEvent)) + 'static;

// An event register.
#[derive(Default)]
pub struct EventRegister {
    register: HashMap<RegisterEvent, Rc<KeyEventCallback>>,
    capture_all: Option<Rc<KeyEventCallback>>,
}

#[derive(Default, Eq, PartialEq, Hash)]
pub(super) enum RegisterEvent {
    #[default]
    All,
    KeyEvent(KeyEvent),
}

impl From<KeyEvent> for RegisterEvent {
    fn from(value: KeyEvent) -> Self {
        Self::KeyEvent(value)
    }
}

impl EventRegister {
    /// Register a new key event.
    pub(crate) fn register_event<T, C>(&mut self, event: T, callback: C)
    where
        T: Into<RegisterEvent>,
        C: Fn((&mut AppContext, KeyEvent)) + 'static,
    {
        self.register.insert(event.into(), Rc::new(callback));
    }

    // /// Registers a capture all callback.
    // pub(crate) fn capture_all<C>(&mut self, callback: C)
    // where
    //     C: Fn(&mut AppContext) + 'static,
    // {
    //     self.capture_all = Some(Rc::new(callback));
    // }

    /// Unregister a key event.
    pub(crate) fn unregister_event<T>(&mut self, event: T)
    where
        T: Into<RegisterEvent>,
    {
        let _ = self.register.remove(&event.into());
    }

    /// Returns a clone event callback.
    pub(crate) fn get_event<T>(&self, event: T) -> Option<Rc<KeyEventCallback>>
    where
        T: Into<RegisterEvent>,
    {
        self.register.get(&event.into()).map(Rc::clone)
    }
}
