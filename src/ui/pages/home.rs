use super::{ListPageState, Page, drop_page, utils::centered_rect};
use crate::ui::{
    app::{AppContext, poll_commands},
    theme::get_theme,
};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::Paragraph,
};
use std::time::Instant;
use tachyonfx::{Duration, Effect, EffectTimer, Interpolation, Shader, fx};

#[derive(Default)]
pub(crate) struct HomePage {}

pub(crate) struct HomePageState {
    intro_effect: Effect,
    last_frame: Instant,
}

impl HomePageState {
    pub(crate) fn new(ctx: &mut AppContext) -> Self {
        let theme = get_theme();
        let bg = theme.base.bg.unwrap_or_default();

        Self::on_mount(ctx);

        Self {
            intro_effect: fx::fade_from_fg(bg, EffectTimer::from_ms(1000, Interpolation::Linear)),
            last_frame: Instant::now(),
        }
    }

    pub(crate) fn on_mount(ctx: &mut AppContext) {
        ctx.notifier.listen("home", |(ctx, key)| {
            if key == KeyEvent::from(KeyCode::Enter) {
                let commands = poll_commands(Duration::from_millis(1000));
                drop_page(ctx);
                ctx.current_page = Page::List(ListPageState::new(ctx, commands));
            }
        });
    }

    pub(crate) fn on_drop(ctx: &mut AppContext) {
        ctx.notifier.unlisten("home");
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

        if state.intro_effect.done() {
            let description = "Search and Browse Man Pages";

            Paragraph::new(description)
                .alignment(Alignment::Center)
                .style(theme.base.italic())
                .render(subtitle, buf);
        } else {
            let frame_duration = state.last_frame.elapsed();
            state.intro_effect.process(frame_duration, buf, title);
        }

        state.last_frame = Instant::now();
    }
}
