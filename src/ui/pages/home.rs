use super::{ListPage, ListPageState, utils::centered_rect};
use crate::ui::{
    app::{ActivePage, ActiveState, AppState},
    events::{Event, EventCtrl, EventStatefulWidget, EventfulWidget},
    theme::get_theme,
};
use ratatui::{
    crossterm::event::KeyCode,
    prelude::*,
    widgets::{Paragraph, StatefulWidgetRef},
};
use std::{cell::RefCell, rc::Rc, time::Instant};
use tachyonfx::{CenteredShrink, Effect, EffectTimer, Interpolation, Shader, fx};

#[derive(Default, Clone)]
pub(crate) struct HomePage {}

pub(crate) struct HomePageState {
    intro_effect: Effect,
    last_frame: Instant,
}

impl EventfulWidget<AppState, Event> for HomePage {
    fn key() -> String {
        String::from("HomePage")
    }

    fn handle_events(
        ctrl: &Rc<RefCell<EventCtrl>>,
        state: &mut AppState,
        event: &Event,
        _: Option<Rect>,
    ) {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Enter {
                let page_state = ListPageState::new(state);
                state.active_state = ActiveState::List(page_state);

                let page = EventStatefulWidget::new(ListPage::new(ctrl), ctrl);
                state.active_page = ActivePage::List(page);
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

impl StatefulWidgetRef for HomePage {
    type State = HomePageState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();
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
