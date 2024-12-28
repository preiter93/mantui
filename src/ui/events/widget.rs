#![allow(dead_code)]
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidgetRef, WidgetRef},
};
use std::{cell::RefCell, rc::Rc};

use super::controller::{EventContext, EventController};

/// A trait that should be implemented by widgets that can handle events.
///
/// Should be used in [`InteractiveWidget`] and [`InteractiveStatefulWidget`].
pub trait EventfulWidget<S, E> {
    /// Returns a unique key for identifying the widget in the event controller.
    ///
    /// This key is used to register and unregister event listeners for the widget.
    fn unique_key() -> String;

    /// Handles incoming events for the widget.
    ///
    /// # Arguments
    /// - `ctx`: The [`EventContext`] containing the [`EventController`] and the event `E`.
    /// - `state`: The mutable application state `S`.
    /// - `area`: The area of the widget's last render.
    fn on_event(ctx: EventContext<S, E>, state: &mut S, area: Option<Rect>);
}

/// A macro to create an `InteractiveWidget` that integrates with an event controller.
macro_rules! interactive_widget {
    ($name:ident, $widget_ref:ident) => {
        /// A wrapper for a widget that integrates with an event controller.
        pub struct $name<S, E, W>
        where
            W: EventfulWidget<S, E> + $widget_ref,
        {
            widget: W,
            controller: EventController<S, E>,
            area: Rc<RefCell<Option<Rect>>>,
        }

        impl<S, E, W> $name<S, E, W>
        where
            S: 'static,
            E: 'static,
            W: EventfulWidget<S, E> + $widget_ref,
        {
            /// Creates a new `$name` instance and registers it as a listener
            /// on the provided `EventController`.
            ///
            /// # Parameters
            /// - `widget`: The widget to wrap in an event handler.
            /// - `controller`: The event controller to register the widget with.
            #[must_use]
            pub fn new(widget: W, controller: &EventController<S, E>) -> Self {
                let area = Rc::new(RefCell::new(None));

                // Clone the controller and register the event handler callback.
                let controller_clone = controller.rc_clone();
                let key = &W::unique_key();
                controller_clone.add_listener(key, {
                    let area_clone = Rc::clone(&area);
                    move |ctx, state| {
                        let area = area_clone.borrow();
                        W::on_event(ctx, state, *area);
                    }
                });

                Self {
                    widget,
                    controller: controller_clone,
                    area,
                }
            }
        }

        impl<S, E, W> Drop for $name<S, E, W>
        where
            W: EventfulWidget<S, E> + $widget_ref,
        {
            /// Removes the listener from the event controller when the widget is dropped.
            fn drop(&mut self) {
                let key = W::unique_key();
                self.controller.remove_listener(&key);
            }
        }
    };
}

interactive_widget!(InteractiveWidget, WidgetRef);

interactive_widget!(InteractiveStatefulWidget, StatefulWidgetRef);

impl<S, E, W> WidgetRef for InteractiveWidget<S, E, W>
where
    W: EventfulWidget<S, E> + WidgetRef,
{
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        *self.area.borrow_mut() = Some(area);
        self.widget.render_ref(area, buf);
    }
}

impl<S, E, W> StatefulWidgetRef for InteractiveStatefulWidget<S, E, W>
where
    W: EventfulWidget<S, E> + StatefulWidgetRef,
{
    type State = <W as StatefulWidgetRef>::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        *self.area.borrow_mut() = Some(area);
        self.widget.render_ref(area, buf, state);
    }
}
