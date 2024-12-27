use anyhow::Result;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use ratatui::crossterm::event::{self, Event as CTEvent, KeyModifiers};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::{app::AppContext, pages::Page};

/// Internal events.
#[derive(Debug, Default)]
pub enum InternalEvent {
    #[default]
    None,
    Loaded(Vec<String>),
}

/// App events.
#[derive(Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,

    /// Key event.
    Key(KeyEvent),

    /// Internal event.
    Internal(InternalEvent),
}

#[allow(dead_code)]
/// An event handler.
pub(crate) struct EventHandler {
    /// Event sender channel.
    pub(crate) sx: mpsc::Sender<Event>,

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
            Event::Internal(event) => {
                // Man commands are loaded in the background.
                //
                // After they are loaded, we assign them to the
                // global AppContext state. And we must also
                // update the command page, if it is open.
                if let InternalEvent::Loaded(commands) = event {
                    ctx.commands = Some(commands.clone());
                    if let Page::List(state) = &mut ctx.current_page {
                        state.commands = Some(commands);
                    }
                }
            }
            Event::Tick => {}
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
