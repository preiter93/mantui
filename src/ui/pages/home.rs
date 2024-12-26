use super::{ManPageState, Page, switch_page, utils::centered_rect};
use crate::ui::{app::AppContext, debug::log_to_file, events::EventHandler, theme::get_theme};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::Paragraph,
};
use std::{collections::HashMap, marker::PhantomData, time::Instant};
use tachyonfx::{Duration, Effect, Shader, fx};

#[derive(Default)]
pub(crate) struct HomePage {}

pub(crate) struct HomePageState {
    title_effect: Effect,
    last_frame: Instant,
}

impl HomePageState {
    pub(crate) fn new(ctx: &mut AppContext, events: &mut EventHandler) -> Self {
        let theme = get_theme();
        let duration = Duration::from_millis(1000);
        let bg = theme.base.bg.unwrap_or_default();

        Self::on_mount(ctx, events);

        Self {
            title_effect: fx::fade_from_fg(bg, duration),
            last_frame: Instant::now(),
        }
    }

    pub(crate) fn on_mount(ctx: &mut AppContext, events: &mut EventHandler) {
        ctx.register
            .register_event(KeyEvent::from(KeyCode::Enter), |ctx| {
                let state = ManPageState::new(ctx);
                switch_page(ctx, Page::Man(state));
            });
    }

    pub(crate) fn on_drop(ctx: &mut AppContext) {
        ctx.register
            .unregister_event(KeyEvent::from(KeyCode::Enter));
    }
}

impl StatefulWidget for HomePage {
    type State = HomePageState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();
        let figlet = r"                       _         _
 _ __ ___   __ _ _ __ | |_ _   _(_)
| '_ ` _ \ / _` | '_ \| __| | | | |
| | | | | | (_| | | | | |_| |_| | |
|_| |_| |_|\__,_|_| |_|\__|\__,_|_|
        ";
        let area = centered_rect(area, 6);

        let [title, subtitle] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .areas(area);

        Paragraph::new(figlet)
            .alignment(Alignment::Center)
            .style(theme.base)
            .render(title, buf);

        if state.title_effect.done() {
            let description = "Search and Browse Man Pages";

            Paragraph::new(description)
                .alignment(Alignment::Center)
                .style(theme.base.italic())
                .render(subtitle, buf);
        } else {
            let frame_duration = state.last_frame.elapsed();
            state.title_effect.process(frame_duration, buf, title);
        }

        state.last_frame = Instant::now();
    }
}
