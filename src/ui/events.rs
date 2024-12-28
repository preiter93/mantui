mod controller;
mod widget;

use super::app::AppState;
use ratatui::crossterm::event::{self, Event as CrosstermEvent};
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use std::thread;
use std::time::{Duration, Instant};
pub(crate) use widget::EventfulWidget;

/// Internal events.
#[derive(Debug, Default)]
pub enum InternalEvent {
    #[default]
    None,
    Loaded((Vec<String>, usize)),
}

/// App events.
#[derive(Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,

    /// Key event.
    Key(KeyEvent),

    /// Mouse event.
    Mouse(MouseEvent),

    /// Internal event.
    Internal(InternalEvent),
}

pub(crate) type EventController = controller::EventController<AppState, Event>;
pub(crate) type EventContext<'a> = controller::EventContext<'a, AppState, Event>;
pub(crate) type IStatefulWidget<W> = widget::InteractiveStatefulWidget<AppState, Event, W>;

pub(crate) fn spawn_event_loop(controller: &EventController, tick_rate_ms: u64) {
    let tick_rate = Duration::from_millis(tick_rate_ms);

    let sender = controller.get_sender();

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // Emit crossterm events
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout).expect("unable to poll events") {
                match event::read().expect("unable to read event") {
                    CrosstermEvent::Key(event) => {
                        let _ = sender.send(Event::Key(event));
                    }
                    CrosstermEvent::Mouse(event) => {
                        let _ = sender.send(Event::Mouse(event));
                    }
                    _ => {}
                }
            }

            // Emit tick events
            if last_tick.elapsed() >= tick_rate {
                let _ = sender.send(Event::Tick);
                last_tick = Instant::now();
            }
        }
    });
}
