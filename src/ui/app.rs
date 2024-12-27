use anyhow::Result;
use ratatui::{crossterm::event::KeyEvent, prelude::*};
use std::{
    error::Error,
    marker::PhantomData,
    sync::OnceLock,
    thread::{self, sleep},
    time::Instant,
};
use tachyonfx::{Duration, Effect, EffectRenderer, Shader, fx};

use crate::core::list_user_commands;

use super::{
    debug::log_to_file,
    events::{Event, EventHandler, EventNotifier},
    pages::{HomePage, HomePageState, ListPage, ManPage, ManPageState, Page},
    terminal::Terminal,
    theme::get_theme,
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
}

impl AppContext {
    fn new() -> Self {
        Self {
            current_page: Page::None,
            should_quit: false,
            notifier: EventNotifier::default(),
            search: String::new(),
            selected_index: None,
        }
    }
}

impl App<'_> {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new() -> App<'static> {
        // The commands take some time to load, thus we load
        // them in the background as soon as the app starts.
        init_commands();

        App {
            ctx: AppContext::new(),
            events: EventHandler::new(100),
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

pub(super) static MAN_COMMANDS: OnceLock<Vec<String>> = OnceLock::new();

fn init_commands() {
    thread::spawn(move || {
        MAN_COMMANDS
            .set(list_user_commands().unwrap_or_default())
            .expect("COMMAND's are already initialized");
    });
}

pub(super) fn poll_commands(timeout: Duration) -> Vec<String> {
    let start = Instant::now();
    let delay = Duration::from_millis(200);

    while start - Instant::now() < timeout {
        if let Some(commands) = MAN_COMMANDS.get() {
            return commands.clone();
        }
        sleep(delay);
    }

    Vec::new()
}
