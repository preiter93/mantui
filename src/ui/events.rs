use std::{
    cell::RefCell,
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

use ratatui::crossterm::event::{self, Event as CTEvent};
use ratatui::crossterm::event::{KeyEvent, MouseEvent};

use super::app::AppState;

use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::{self};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};

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

pub(crate) type EventCtrl = EventController<AppState, Event>;
pub(crate) type EventCtrlRc = EventControllerRc<AppState, Event>;

/// Emits regular tick events in a separate thread.
pub(crate) fn emit_events(ctrl: &EventCtrl, tick_rate_ms: u64) {
    let tick_rate = Duration::from_millis(tick_rate_ms);

    let sender = ctrl.sender.clone();

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // Emit key events
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout).expect("unable to poll events") {
                match event::read().expect("unable to read event") {
                    CTEvent::Key(event) => {
                        let _ = sender.send(Event::Key(event));
                    }
                    CTEvent::Mouse(event) => {
                        // if let MouseEventKind::Down(_) = event.kind {
                        // }
                        let _ = sender.send(Event::Mouse(event));
                    }
                    _ => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                // Emit tick events
                let _ = sender.send(Event::Tick);
                last_tick = Instant::now();
            }
        }
    });
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

type EventCallback<S, E> = Rc<dyn Fn(&Rc<RefCell<EventController<S, E>>>, &mut S, &E) + 'static>;

/// An event controller for ratatui apps.
///
/// ```
/// use ratatui::prelude::*;
/// use ratatui::widgets::WidgetRef;
/// use tui_event_controller::{EventWidget, EventfulWidget};
/// use tui_event_controller::{EventController, EventControllerRc};
///
/// #[derive(Default)]
/// struct AppState {
///     counter: usize,
/// }
///
/// /// App events.
/// #[derive(Debug)]
/// enum AppEvent {
///     Tick,
/// }
///
/// type AppEventController = EventController<AppState, AppEvent>;
/// type AppEventControllerRc = EventControllerRc<AppState, AppEvent>;
///
/// #[derive(Default)]
/// struct MyWidget {}
///
/// impl WidgetRef for MyWidget {
///     fn render_ref(&self, area: Rect, buf: &mut Buffer) {}
/// }
///
/// impl MyWidget {
///     const KEY: &str = "foo";
///
///     fn new() -> Self {
///         Self {}
///     }
/// }
///
/// impl EventfulWidget<AppState, AppEvent> for MyWidget {
///     fn key() -> String {
///         String::from("MyWidget")
///     }
///
///     fn handle_events(
///         ctrl: &AppEventControllerRc,
///         state: &mut AppState,
///         event: &AppEvent,
///         area: Option<Rect>,
///      ) {
///         state.counter += 1;
///         println!("event: {event:?} | counter: {}", state.counter);
///     }
/// }
///
/// let mut state = AppState::default();
/// let mut event_ctrl = AppEventController::new();
///
/// let widget = MyWidget::default();
/// let widget = EventWidget::new(widget, &event_ctrl);
/// ```
pub struct EventController<S, E> {
    /// Event sender channel.
    pub sender: mpsc::Sender<E>,

    /// Event receiver channel.
    receiver: mpsc::Receiver<E>,

    /// Registered callbacks.
    callbacks: HashMap<String, EventCallback<S, E>>,
}

/// A typedef for an event controller in a rc pointer.
pub type EventControllerRc<S, E> = Rc<RefCell<EventController<S, E>>>;

impl<S, E> EventController<S, E> {
    /// Creates a new instance of [`EventHandler`].
    #[must_use]
    pub fn new() -> Rc<RefCell<Self>> {
        let (sender, receiver) = mpsc::channel();
        let callbacks = HashMap::default();

        Rc::new(RefCell::new(Self {
            sender,
            receiver,
            callbacks,
        }))
    }

    /// Adds a new listener.
    ///
    /// # Example
    /// ```
    /// use tui_event_controller::EventController;
    ///
    /// struct AppState;
    /// #[derive(Debug)]
    /// struct AppEvent;
    ///
    /// let mut event_ctrl = EventController::<AppState, AppEvent>::new();
    /// event_ctrl.borrow_mut().add_listener("foo", move |_ctrl, _state, event| {
    ///     println!("received: {event:?}");
    /// });
    /// ```
    pub fn add_listener<F>(&mut self, id: &str, callback: F)
    where
        F: Fn(&Rc<RefCell<EventController<S, E>>>, &mut S, &E) + 'static,
    {
        self.callbacks.insert(id.to_string(), Rc::new(callback));
    }

    /// Removes a listener.
    ///
    /// # Example
    /// ```
    /// use tui_event_controller::EventController;
    ///
    /// struct AppState;
    /// struct AppEvent;
    ///
    /// let mut event_ctrl = EventController::<AppState, AppEvent>::new();
    /// event_ctrl.borrow_mut().remove_listener("foo");
    /// ```
    pub fn remove_listener(&mut self, id: &str) {
        let _ = self.callbacks.remove(id);
    }
}

/// Waits for events and notifies all listeners.
///
/// # Errors
///
/// Returns an error if the channel has hang up.
///
/// # Example
/// ```ignore
/// use tui_event_controller::EventController;
///
/// struct AppState;
/// struct AppEvent;
///
/// let mut state = AppState;
/// let mut event_ctrl = EventController::<AppState, AppEvent>::new();
///
/// event_ctrl
///     .borrow_mut()
///     .handle_events(&mut state)
///     .expect("failed to handle events");
/// ```
pub fn handle_events<S, E>(s: &Rc<RefCell<EventController<S, E>>>, state: &mut S) -> Result<()> {
    let event = s.borrow().receiver.recv()?;

    let callbacks = s.borrow().callbacks.clone();
    for callback in callbacks.values() {
        (callback)(s, state, &event);
    }

    Ok(())
}

/// A trait for widgets that can handle events.
pub trait EventfulWidget<S, E> {
    /// Returns a unique key for identifying the widget in the event controller.
    fn key() -> String;

    /// Handles incoming events for the widget.
    ///
    /// # Arguments
    /// - `state`: A mutable reference to the state associated with the widget.
    /// - `event`: The event to be processed.
    /// - `area`: The area of the widget from the last render.
    fn handle_events(
        ctrl: &Rc<RefCell<EventController<S, E>>>,
        state: &mut S,
        event: &E,
        area: Option<Rect>,
    );
}

/// A wrapper for a widget that integrates with an event controller.
///
/// This automatically registers a listener on the `EventController`
/// when constructed with `EventfulWidget::new` and removes the listener
/// when it is dropped.
pub struct EventWidget<S, E, W>
where
    W: EventfulWidget<S, E>,
    W: WidgetRef,
{
    widget: W,
    ctrl: Rc<RefCell<EventController<S, E>>>,
    area: Rc<RefCell<Option<Rect>>>,
}

impl<S, E, W> EventWidget<S, E, W>
where
    S: 'static,
    E: 'static,
    W: EventfulWidget<S, E>,
    W: WidgetRef,
{
    /// Creates a new instance of `EventfulWidget` and registers a
    /// callback on the the event controller.
    #[must_use]
    pub fn new(widget: W, controller: &Rc<RefCell<EventController<S, E>>>) -> Self {
        let area = Rc::new(RefCell::new(None));
        let area_clone = Rc::clone(&area);

        let ctrl = Rc::clone(controller);

        let key = &W::key();
        ctrl.borrow_mut()
            .add_listener(key, move |ctrl, state, event| {
                let area = area_clone.borrow();
                W::handle_events(ctrl, state, event, *area);
            });

        Self { widget, ctrl, area }
    }
}

impl<S, E, W> Drop for EventWidget<S, E, W>
where
    W: EventfulWidget<S, E>,
    W: WidgetRef,
{
    /// Removes the listener from the event controller when it is dropped.
    fn drop(&mut self) {
        let key = &W::key();
        self.ctrl.borrow_mut().remove_listener(key);
    }
}

impl<S, E, W> WidgetRef for &mut EventWidget<S, E, W>
where
    W: EventfulWidget<S, E>,
    W: WidgetRef,
{
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        *self.area.borrow_mut() = Some(area);
        self.widget.render_ref(area, buf);
    }
}

pub struct EventStatefulWidget<S, E, W>
where
    W: EventfulWidget<S, E>,
    W: StatefulWidgetRef,
{
    widget: W,
    ctrl: Rc<RefCell<EventController<S, E>>>,
    area: Rc<RefCell<Option<Rect>>>,
}

impl<S, E, W> EventStatefulWidget<S, E, W>
where
    S: 'static,
    E: 'static,
    W: EventfulWidget<S, E>,
    W: StatefulWidgetRef,
{
    /// Creates a new instance of `EventfulWidget` and registers a
    /// callback on the the event controller.
    #[must_use]
    pub fn new(widget: W, controller: &Rc<RefCell<EventController<S, E>>>) -> Self {
        let area = Rc::new(RefCell::new(None));
        let area_clone = Rc::clone(&area);

        let ctrl = Rc::clone(controller);

        let key = &W::key();
        ctrl.borrow_mut()
            .add_listener(key, move |ctrl, state, event| {
                let area = area_clone.borrow();
                W::handle_events(ctrl, state, event, *area);
            });

        Self { widget, ctrl, area }
    }
}

impl<S, E, W> Drop for EventStatefulWidget<S, E, W>
where
    W: EventfulWidget<S, E>,
    W: StatefulWidgetRef,
{
    /// Removes the listener from the event controller when it is dropped.
    fn drop(&mut self) {
        let key = &W::key();
        self.ctrl.borrow_mut().remove_listener(key);
    }
}

impl<S, E, W> StatefulWidgetRef for EventStatefulWidget<S, E, W>
where
    W: EventfulWidget<S, E>,
    W: StatefulWidgetRef,
{
    type State = <W as StatefulWidgetRef>::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        *self.area.borrow_mut() = Some(area);
        self.widget.render_ref(area, buf, state);
    }
}
