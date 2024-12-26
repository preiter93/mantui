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
                if let Some(callback) = ctx.register.get_event(&key) {
                    (callback)(ctx);
                }
            }
        }

        Ok(())
    }
}

type KeyEventCallback = dyn Fn(&mut AppContext) + 'static;

// An event register.
#[derive(Default)]
pub struct EventRegister {
    register: HashMap<KeyEvent, Rc<KeyEventCallback>>,
}

impl EventRegister {
    /// Register a new key event.
    pub(crate) fn register_event<C>(&mut self, event: KeyEvent, callback: C)
    where
        C: Fn(&mut AppContext) + 'static,
    {
        self.register.insert(event, Rc::new(callback));
    }

    /// Unregister a key event.
    pub(crate) fn unregister_event(&mut self, event: KeyEvent) {
        let _ = self.register.remove(&event);
    }

    /// Returns a clone event callback.
    pub(crate) fn get_event(&self, event: &KeyEvent) -> Option<Rc<KeyEventCallback>> {
        self.register.get(event).map(Rc::clone)
    }
}
