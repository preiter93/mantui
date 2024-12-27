use anyhow::Result;
use ratatui::prelude::*;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex, mpsc},
    thread::{self},
    time::Duration,
};
use uuid::Uuid;

use crate::core::load_section;

use super::{
    events::{Event, EventHandler, EventNotifier, InternalEvent},
    pages::{HomePage, HomePageState, ListPage, ManPage, Page},
    terminal::Terminal,
};

pub struct App<'a> {
    pub(super) ctx: AppContext,
    _phantom: PhantomData<&'a ()>,
    pub(super) events: EventHandler,
}

pub(super) struct AppContext {
    pub(super) should_quit: bool,
    pub(super) current_page: Page,
    pub(super) notifier: EventNotifier,
    pub(super) search: String,
    pub(super) selected_command: Option<usize>,
    pub(super) selected_section: usize,
    pub(super) commands: Option<Vec<String>>,
    pub(crate) sx: mpsc::Sender<Event>,
    debouncer: Arc<Mutex<Uuid>>,
}

impl AppContext {
    fn new(sx: mpsc::Sender<Event>) -> Self {
        Self {
            current_page: Page::None,
            should_quit: false,
            notifier: EventNotifier::default(),
            search: String::new(),
            selected_command: None,
            selected_section: 0,
            commands: None,
            sx,
            debouncer: Arc::new(Mutex::new(Uuid::new_v4())),
        }
    }
}

impl App<'_> {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new() -> App<'static> {
        let events = EventHandler::new(100);
        let ctx = AppContext::new(events.sx.clone());

        // Loading the man commands takes some time,
        // thuse they are loaded in the background.
        load_commands_in_background(&ctx, 0);

        App {
            ctx,
            events,
            _phantom: PhantomData,
        }
    }

    pub fn run() -> Result<()> {
        let mut terminal = Terminal::new()?;
        let mut app = Self::new();

        let state = HomePageState::new(&mut app.ctx);
        app.ctx.current_page = Page::Home(state);

        while !app.ctx.should_quit {
            terminal.draw(|frame| {
                frame.render_widget(&mut app, frame.area());
            })?;
            app.events.handle(&mut app.ctx)?;
        }

        Terminal::stop()?;
        Ok(())
    }
}

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &mut self.ctx.current_page {
            Page::Home(state) => {
                let page = HomePage::default();
                page.render(area, buf, state);
            }
            Page::List(state) => {
                let page = ListPage::default();
                page.render(area, buf, state);
            }
            Page::Desc(state) => {
                let page = ManPage::default();
                page.render(area, buf, state);
            }
            Page::None => {}
        }
    }
}

pub(crate) fn load_commands_in_background(ctx: &AppContext, section: usize) {
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
