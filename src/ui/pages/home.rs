use super::{utils::centered_rect, ListPage, ListPageState};
use crate::ui::{
    app::{ActiveState, ActiveWidget, AppState},
    events::{Event, EventContext, EventController, EventfulWidget, IStatefulWidget},
    theme::get_theme,
};
use ratatui::{
    crossterm::event::{KeyCode, MouseEventKind},
    prelude::*,
    widgets::{Paragraph, StatefulWidgetRef},
};
use std::time::Instant;
use tachyonfx::{fx, CenteredShrink, Effect, EffectTimer, Interpolation};

#[derive(Default, Clone)]
pub(crate) struct HomePage {}

pub(crate) struct HomePageState {
    intro_effect: Effect,
    last_frame: Instant,
}

impl EventfulWidget<AppState, Event> for HomePage {
    fn unique_key() -> String {
        String::from("HomePage")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, _: Option<Rect>) {
        if let Event::Key(key) = ctx.event {
            if key.code == KeyCode::Enter {
                next_page(ctx.controller, state);
            }
        }
        if let Event::Mouse(event) = ctx.event {
            if let MouseEventKind::Down(_) = event.kind {
                next_page(ctx.controller, state);
            }
        }
    }
}

impl HomePageState {
    pub(crate) fn new() -> Self {
        let theme = get_theme();
        let bg = theme.base.bg.unwrap_or_default();

        Self {
            intro_effect: fx::fade_from_fg(bg, EffectTimer::from_ms(1000, Interpolation::Linear)),
            last_frame: Instant::now(),
        }
    }
}

fn next_page(controller: &EventController, state: &mut AppState) {
    let page_state = ListPageState::new(state);
    state.active_state = ActiveState::List(page_state);

    let page = ListPage::new(controller);
    let page = IStatefulWidget::new(page, controller);
    state.active_page = ActiveWidget::List(page);
}

impl StatefulWidgetRef for HomePage {
    type State = HomePageState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();
        buf.set_style(area, theme.base);

        let figlet = r"                       _         _
 _ __ ___   __ _ _ __ | |_ _   _(_)
| '_ ` _ \ / _` | '_ \| __| | | | |
| | | | | | (_| | | | | |_| |_| | |
|_| |_| |_|\__,_|_| |_|\__|\__,_|_|
        ";
        let centered_area = centered_rect(area, 8);

        let [main, instruction] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .areas(centered_area);

        let [title, subtitle] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(2)])
            .areas(main);

        Paragraph::new(figlet)
            .alignment(Alignment::Center)
            .style(theme.base)
            .render(title, buf);

        Line::from("Search and Browse Man Pages")
            .alignment(Alignment::Center)
            .style(theme.base.italic())
            .render(subtitle, buf);

        if state.intro_effect.done() {
            let instruction_text = InstructionText;
            instruction_text.render(instruction.inner_centered(23, 1), buf, state);
        } else {
            let frame_duration = state.last_frame.elapsed();
            state.intro_effect.process(frame_duration, buf, main);
            state.last_frame = Instant::now();
        }
    }
}

struct InstructionText;

impl StatefulWidget for InstructionText {
    type State = HomePageState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let theme = get_theme();

        let [l, m, r] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .areas(area);
        Line::from("Press ").style(theme.base).render(l, buf);
        Line::from("Enter").style(theme.base.bold()).render(m, buf);
        Line::from(" to Continue").style(theme.base).render(r, buf);
    }
}
