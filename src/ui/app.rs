use anyhow::Result;
use ratatui::widgets::StatefulWidgetRef;
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    prelude::*,
};
use std::{
    marker::PhantomData,
    sync::{mpsc, Arc, Mutex},
    thread::{self},
    time::Duration,
};
use uuid::Uuid;

use crate::core::load_section;

use super::debug::log_to_file;
use super::events::{EventController, IStatefulWidget};
use super::pages::{ReaderPage, ReaderPageState};
use super::{
    events::{spawn_event_loop, Event, InternalEvent},
    pages::{HomePage, HomePageState, ListPage, ListPageState},
    terminal::Terminal,
};

pub struct App<'a> {
    _phantom: PhantomData<&'a ()>,
}

pub(crate) enum Navigation {
    List,
    Reader,
}

impl Navigation {
    pub(crate) fn navigate_to(to: &Self, app_state: &mut AppState, controller: &EventController) {
        match to {
            Navigation::List => {
                let state = ListPageState::new(app_state);
                app_state.active_state = ActiveState::List(state);

                let page = IStatefulWidget::new(ListPage::new(controller), controller);
                app_state.active_page = ActiveWidget::List(page);
            }
            Navigation::Reader => {
                let ActiveState::List(old_state) = &mut app_state.active_state else {
                    return;
                };
                if let Some(command) = old_state.selected_command() {
                    // Store the selected command for later in case we navigate
                    // back again from the reader to the list
                    app_state.selected_command = old_state.selected_command_index();
                    app_state.selected_section = old_state.selected_section_index();
                    app_state.command_search = old_state.command_search();
                    app_state.loaded_commands = old_state.loaded_commands();

                    let state = ReaderPageState::new(&command, old_state.page_width());
                    app_state.active_state = ActiveState::Read(state);

                    let page = IStatefulWidget::new(ReaderPage::new(controller), controller);
                    app_state.active_page = ActiveWidget::Read(page);
                }
            }
        }
    }
}

pub(crate) enum ActiveWidget {
    Home(IStatefulWidget<HomePage>),
    List(IStatefulWidget<ListPage>),
    Read(IStatefulWidget<ReaderPage>),
}

pub(crate) enum ActiveState {
    Home(HomePageState),
    List(ListPageState),
    Read(ReaderPageState),
}

pub struct AppState {
    pub(super) should_quit: bool,
    pub(super) active_page: ActiveWidget,
    pub(super) active_state: ActiveState,

    pub(super) selected_command: Option<usize>,
    pub(super) selected_section: usize,
    pub(super) loaded_commands: Option<Vec<String>>,

    pub(super) command_search: String,

    pub(crate) sx: mpsc::Sender<Event>,
    debouncer: Arc<Mutex<Uuid>>,
}

impl AppState {
    pub(super) fn new(
        initial_command: Option<String>,
        initial_area: Rect,
        controller: &EventController,
    ) -> Self {
        let (active_page, active_state) = if let Some(command) = initial_command {
            let page = ReaderPage::new(controller);
            let state = ReaderPageState::new(&command, initial_area.width as usize);
            (
                ActiveWidget::Read(IStatefulWidget::new(page, controller)),
                ActiveState::Read(state),
            )
        } else {
            let page = HomePage {};
            let state = HomePageState::new();
            (
                ActiveWidget::Home(IStatefulWidget::new(page, controller)),
                ActiveState::Home(state),
            )
        };

        Self {
            should_quit: false,
            active_page,
            active_state,
            selected_command: None,
            selected_section: 0,
            loaded_commands: None,
            command_search: String::new(),
            sx: controller.get_sender(),
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

    pub fn run(initial_command: Option<String>) -> Result<()> {
        let mut terminal = Terminal::new()?;
        let initial_area = terminal.area();

        let controller = EventController::new();
        spawn_event_loop(&controller, 50);

        let mut app = Self::new();
        let mut state = AppState::new(initial_command, initial_area, &controller);

        // Register global events.
        register_global_events(&controller);

        // Loading the man commands takes some time,
        // thuse they are loaded in the background.
        load_commands_in_background(&state, 0);

        while !state.should_quit {
            terminal.draw(|frame| {
                frame.render_stateful_widget(&mut app, frame.area(), &mut state);
            })?;
            controller.recv_and_notify(&mut state)?;
        }

        Terminal::stop()?;
        Ok(())
    }
}

pub fn register_global_events(controller: &EventController) {
    controller.add_listener("main", |ctx, state| match ctx.event {
        Event::Key(key) => {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                state.should_quit = true;
            }
        }
        Event::Internal(InternalEvent::Loaded((commands, section))) => {
            if let ActiveState::List(state) = &mut state.active_state {
                log_to_file(format!(
                    "{:?} {:?}",
                    section,
                    state.selected_section_index()
                ));
                if state.selected_section_index() == *section {
                    state.set_loaded_commands(commands);
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
            (ActiveWidget::Home(page), ActiveState::Home(state)) => {
                page.render_ref(area, buf, state);
            }
            (ActiveWidget::List(page), ActiveState::List(state)) => {
                page.render_ref(area, buf, state);
            }
            (ActiveWidget::Read(page), ActiveState::Read(state)) => {
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
        log_to_file("load section");
        let commands = load_section(section_str).unwrap_or_default();
        log_to_file("load section done");

        // Send the result
        let event = InternalEvent::Loaded((commands, section));
        let _ = sx1.send(Event::Internal(event));
    });
}
