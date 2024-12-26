use anyhow::Result;
use ratatui::{crossterm::event::KeyEvent, prelude::*};
use std::{error::Error, marker::PhantomData, time::Instant};
use tachyonfx::{Duration, Effect, EffectRenderer, Shader, fx};

use super::{
    debug::log_to_file,
    events::{Event, EventHandler, EventRegister},
    pages::{HomePage, HomePageState, ManPage, Page},
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
    pub(super) register: EventRegister,
}

impl AppContext {
    fn new() -> Self {
        Self {
            current_page: Page::None,
            should_quit: false,
            register: EventRegister::default(),
        }
    }
}

impl App<'_> {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new() -> App<'static> {
        App {
            ctx: AppContext::new(),
            events: EventHandler::new(100),
            _phantom: PhantomData,
        }
    }

    pub fn run() -> Result<()> {
        let mut terminal = Terminal::new()?;
        let mut app = Self::new();

        let state = HomePageState::new(&mut app.ctx, &mut app.events);
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
            Page::Man(state) => {
                let page = ManPage::default();
                page.render(area, buf, state);
            }
            Page::None => {}
        }
    }
}
