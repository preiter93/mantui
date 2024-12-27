use anyhow::Result;
use std::{
    collections::{HashMap, hash_map::Iter},
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
            Event::Tick => {}
            Event::Key(key) => {
                // Always quit on <ctrl-c>.
                if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                    ctx.should_quit = true;
                    return Ok(());
                }

                // Notify all registered listeners
                let notifier = ctx.notifier.clone();
                notifier.notify_listener(ctx, key);
            }
        }

        Ok(())
    }
}

type KeyEventCallback = dyn Fn((&mut AppContext, KeyEvent)) + 'static;

// An event register.
#[derive(Default, Clone)]
pub struct EventNotifier {
    pub(super) listeners: HashMap<String, Rc<KeyEventCallback>>,
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

impl EventNotifier {
    /// Register a new listener.
    pub(crate) fn listen<T, C>(&mut self, id: T, callback: C)
    where
        T: Into<String>,
        C: Fn((&mut AppContext, KeyEvent)) + 'static,
    {
        self.listeners.insert(id.into(), Rc::new(callback));
    }

    /// Unregister a listener.
    pub(crate) fn unlisten<T>(&mut self, id: T)
    where
        T: Into<String>,
    {
        let _ = self.listeners.remove(&id.into());
    }

    /// Notifies all listener.
    fn notify_listener(&self, ctx: &mut AppContext, key: KeyEvent) {
        for callback in self.listeners.values() {
            (callback)((ctx, key));
        }
    }
}
