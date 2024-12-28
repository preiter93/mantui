use anyhow::Result;
use ratatui::widgets::StatefulWidgetRef;
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    prelude::*,
};
use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
    thread::{self},
    time::Duration,
};
use uuid::Uuid;

use crate::core::load_section;

use super::events::{EventCtrlRc, EventStatefulWidget, handle_events};
use super::pages::{ManPage, ManPageState};
use super::{
    events::{Event, EventCtrl, InternalEvent, emit_events},
    pages::{HomePage, HomePageState, ListPage, ListPageState},
    terminal::Terminal,
};

pub struct App<'a> {
    _phantom: PhantomData<&'a ()>,
}

pub enum ActivePage {
    Home(EventStatefulWidget<AppState, Event, HomePage>),
    List(EventStatefulWidget<AppState, Event, ListPage>),
    Man(EventStatefulWidget<AppState, Event, ManPage>),
}
pub enum ActiveState {
    Home(HomePageState),
    List(ListPageState),
    Man(ManPageState),
}

pub struct AppState {
    pub(super) should_quit: bool,
    pub(super) active_page: ActivePage,
    pub(super) active_state: ActiveState,

    pub(super) selected_command: Option<usize>,
    pub(super) selected_section: usize,
    pub(super) commands: Option<Vec<String>>,

    pub(super) search: String,

    pub(crate) sx: mpsc::Sender<Event>,
    debouncer: Arc<Mutex<Uuid>>,
}

impl AppState {
    pub(super) fn new(controller: &Rc<RefCell<EventCtrl>>) -> Self {
        let page = HomePage {};
        let state = HomePageState::new();

        Self {
            should_quit: false,
            active_page: ActivePage::Home(EventStatefulWidget::new(page, controller)),
            active_state: ActiveState::Home(state),
            selected_command: None,
            selected_section: 0,
            commands: None,
            search: String::new(),
            sx: controller.borrow().sender.clone(),
            debouncer: Arc::new(Mutex::new(Uuid::new_v4())),
        }
    }
}

impl App<'_> {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new() -> App<'static> {
        App {
            _phantom: PhantomData,
        }
    }

    pub fn run() -> Result<()> {
        let mut terminal = Terminal::new()?;
        let mut app = Self::new();

        let controller = EventCtrl::new();
        let mut state = AppState::new(&controller);
        emit_events(&controller.borrow(), 50);

        // Register global events.
        register_global_events(&controller);

        // Loading the man commands takes some time,
        // thuse they are loaded in the background.
        load_commands_in_background(&state, 0);

        while !state.should_quit {
            terminal.draw(|frame| {
                frame.render_stateful_widget(&mut app, frame.area(), &mut state);
            })?;
            handle_events(&controller, &mut state).unwrap();
        }

        Terminal::stop()?;
        Ok(())
    }
}

pub fn register_global_events(controller: &EventCtrlRc) {
    controller
        .borrow_mut()
        .add_listener("main", |_, state, event| match event {
            Event::Key(key) => {
                if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                    state.should_quit = true;
                }
            }
            Event::Internal(InternalEvent::Loaded((commands, section))) => {
                if state.selected_section == *section {
                    state.commands = Some(commands.clone());
                    if let ActiveState::List(state) = &mut state.active_state {
                        state.commands = Some(commands.clone());
                    }
                }
            }
            _ => {}
        });
}

impl StatefulWidget for &mut App<'_> {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match (&state.active_page, &mut state.active_state) {
            (ActivePage::Home(page), ActiveState::Home(state)) => {
                page.render_ref(area, buf, state);
            }
            (ActivePage::List(page), ActiveState::List(state)) => {
                page.render_ref(area, buf, state);
            }
            (ActivePage::Man(page), ActiveState::Man(state)) => {
                page.render_ref(area, buf, state);
            }
            _ => {}
        }
    }
}

pub(crate) fn load_commands_in_background(ctx: &AppState, section: usize) {
    let uuid = Uuid::new_v4();

    let sx1 = ctx.sx.clone();

    let section_str = (section + 1).to_string();

    let debouncer_clone = ctx.debouncer.clone();
    let debounce_time = Duration::from_millis(200);

    thread::spawn(move || {
        {
            let mut last_uuid = debouncer_clone.lock().unwrap();
            *last_uuid = uuid;
        }

        // Sleep for a short time to await new requests
        thread::sleep(debounce_time);

        // Only proceed if the last called uuid matches with
        // the uuid of this process.
        {
            let last_uuid = debouncer_clone.lock().unwrap();
            if *last_uuid != uuid {
                return;
            }
        }

        // Load the commands after the debounce check
        let commands = load_section(section_str).unwrap_or_default();

        // Send the result
        let event = InternalEvent::Loaded((commands, section));
        let _ = sx1.send(Event::Internal(event));
    });
}
