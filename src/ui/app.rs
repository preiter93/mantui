use anyhow::Result;
use ratatui::prelude::*;
use std::{
    marker::PhantomData,
    thread::{self},
};

use crate::core::list_user_commands;

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
    pub(super) selected_index: Option<usize>,
    pub(super) commands: Option<Vec<String>>,
}

impl AppContext {
    fn new() -> Self {
        Self {
            current_page: Page::None,
            should_quit: false,
            notifier: EventNotifier::default(),
            search: String::new(),
            selected_index: None,
            commands: None,
        }
    }
}

impl App<'_> {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new() -> App<'static> {
        // The commands take some time to load, thus we load
        // them in the background as soon as the app starts.
        let events = EventHandler::new(100);
        load_commands(&events);

        App {
            ctx: AppContext::new(),
            events,
            _phantom: PhantomData,
        }
    }

    pub fn run() -> Result<()> {
        let mut terminal = Terminal::new()?;
        let mut app = Self::new();

        let state = HomePageState::new(&mut app.ctx);
        app.ctx.current_page = Page::Home(state);
        // let state = DescPageState::new(&mut app.ctx, "grep", 50);
        // app.ctx.current_page = Page::Desc(state);

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

fn load_commands(events: &EventHandler) {
    let sx1 = events.sx.clone();
    thread::spawn(move || {
        let commands = list_user_commands().unwrap_or_default();
        let _ = sx1.send(Event::Internal(InternalEvent::Loaded(commands)));
    });
}
