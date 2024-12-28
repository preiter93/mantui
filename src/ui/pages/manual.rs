#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
use std::cmp::min;

use crate::{
    core::get_manual,
    ui::{
        app::{ActivePage, ActiveState, AppState},
        events::{Event, EventCtrlRc, EventStatefulWidget, EventfulWidget},
        theme::get_theme,
    },
};
use ansi_to_tui::IntoText;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers, MouseEventKind},
    prelude::*,
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidgetRef,
    },
};

use super::{ListPage, ListPageState};

pub(crate) struct ManPage {
    paragraph: EventStatefulWidget<AppState, Event, Content>,
}

impl ManPage {
    pub(crate) fn new(controller: &EventCtrlRc) -> Self {
        Self {
            paragraph: EventStatefulWidget::new(Content {}, controller),
        }
    }
}

impl EventfulWidget<AppState, Event> for ManPage {
    fn key() -> String {
        String::from("ManPage")
    }

    fn handle_events(ctrl: &EventCtrlRc, ctx: &mut AppState, event: &Event, _: Option<Rect>) {
        let ActiveState::Man(state) = &mut ctx.active_state else {
            return;
        };

        if let Event::Key(event) = event {
            match event.code {
                KeyCode::Char(ch)
                    if state.search_active && event.modifiers != KeyModifiers::CONTROL =>
                {
                    state.selected_match = None;
                    state.search.push(ch);
                    state.matches = find_matches_positions(&state.text, &state.search);
                    state.select_next_search();
                }
                KeyCode::Char('j') => {
                    state.scroll_up();
                }
                KeyCode::Char('k') => {
                    state.scroll_down();
                }
                KeyCode::Char('d') if event.modifiers == KeyModifiers::CONTROL => {
                    state.scroll_pos = min(
                        state.scroll_pos + state.page_height / 2,
                        state.max_scroll_pos,
                    );
                }
                KeyCode::Char('u') if event.modifiers == KeyModifiers::CONTROL => {
                    state.scroll_pos = state.scroll_pos.saturating_sub(state.page_height / 2);
                }
                KeyCode::Char('G') if event.modifiers == KeyModifiers::SHIFT => {
                    state.scroll_pos = state.max_scroll_pos;
                }
                KeyCode::Char('g') => {
                    state.scroll_pos = 0;
                }

                KeyCode::Char('N') if event.modifiers == KeyModifiers::SHIFT => {
                    state.select_previous_search();
                }
                KeyCode::Char('n') => {
                    state.select_next_search();
                }
                KeyCode::Char('/') => {
                    state.search_active = true;
                }
                KeyCode::Backspace if state.search_active => {
                    state.search.pop();
                }
                KeyCode::Esc => {
                    if state.search_active {
                        state.search_active = false;
                    } else if state.search.is_empty() {
                        let page_state = ListPageState::new(ctx);
                        ctx.active_state = ActiveState::List(page_state);

                        let page = EventStatefulWidget::new(ListPage {}, ctrl);
                        ctx.active_page = ActivePage::List(page);
                    } else {
                        state.search = String::new();
                        state.matches = Vec::new();
                        state.selected_match = None;
                    }
                }
                KeyCode::Enter if state.search_active => {
                    state.search_active = false;
                }
                _ => {}
            }
        };
    }
}

#[derive(Default)]
pub(crate) struct ManPageState {
    text: Text<'static>,
    scroll_pos: usize,
    page_height: usize,
    max_scroll_pos: usize,
    scrollbar: ScrollbarState,
    search_active: bool,
    search: String,
    selected_match: Option<usize>,
    matches: Vec<(u16, u16)>,
}

impl ManPageState {
    fn scroll_up(&mut self) {
        self.scroll_pos = self.scroll_pos.saturating_sub(1);
    }
    fn scroll_down(&mut self) {
        self.scroll_pos = min(self.scroll_pos + 1, self.max_scroll_pos);
    }

    pub fn select_next_search(&mut self) {
        if self.matches.is_empty() {
            return;
        }

        let padding = 2;

        self.selected_match = self.selected_match.map_or(Some(0), |selected| {
            Some(min(selected + 1, self.matches.len().saturating_sub(1)))
        });

        if let Some(selected_index) = self.selected_match {
            let (selected_row, _) = self.matches[selected_index];
            let selected_row = selected_row as usize;

            // Check if the selected match is after the visible range
            let last_visible_row = self.scroll_pos + self.page_height.saturating_sub(3) - padding;
            let diff = selected_row.saturating_sub(last_visible_row);
            if diff > 0 {
                self.scroll_pos += diff;
                self.scroll_pos = self.scroll_pos.min(self.max_scroll_pos);
            }

            // Check if the selected match is above the visible range
            let first_visible_row = self.scroll_pos + padding;
            if selected_row < first_visible_row {
                self.scroll_pos = selected_row.saturating_sub(padding);
            }
        }
    }

    pub fn select_previous_search(&mut self) {
        if self.matches.is_empty() {
            return;
        }

        let padding = 2;

        self.selected_match = self
            .selected_match
            .map_or(Some(0), |selected| Some(selected.saturating_sub(1)));

        if let Some(selected_index) = self.selected_match {
            let (selected_row, _) = self.matches[selected_index];
            let selected_row = selected_row as usize;

            // Check if the selected match is after the visible range
            let last_visible_row = self.scroll_pos + self.page_height.saturating_sub(3) - padding;
            let diff = selected_row.saturating_sub(last_visible_row);
            if diff > 0 {
                self.scroll_pos += diff;
                self.scroll_pos = self.scroll_pos.min(self.max_scroll_pos);
            }

            // Check if the selected match is above the visible range
            let first_visible_row = self.scroll_pos + padding;
            if selected_row < first_visible_row {
                self.scroll_pos = selected_row.saturating_sub(padding);
            }
        }
    }

    fn selected_match(&self) -> Option<(u16, u16)> {
        if let Some(index) = self.selected_match {
            return Some(self.matches[index]);
        }

        None
    }
}

impl ManPageState {
    pub(crate) fn new(command: &str, width: usize) -> Self {
        let reduced_width = (width as f64 * 0.9) as u16;

        let width = format!("{reduced_width}");
        let text_raw = get_manual(command, &width).unwrap_or_default();

        let text =
            IntoText::into_text(&text_raw).unwrap_or(Text::from("Could not convert ansi to tui."));

        let scrollbar = ScrollbarState::new(0).position(0);

        Self {
            scroll_pos: 0,
            page_height: 0,
            max_scroll_pos: 0,
            text,
            scrollbar,
            search: String::new(),
            matches: Vec::new(),
            selected_match: None,
            search_active: false,
        }
    }
}

impl StatefulWidgetRef for ManPage {
    type State = ManPageState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [main, search] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .areas(area);

        let theme = get_theme();

        let style = if state.search_active {
            theme.block.inactive
        } else {
            theme.block.active
        };

        let block = Block::default()
            .style(style)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .padding(Padding::horizontal(1));
        let inner = block.inner(main);
        block.render(main, buf);

        // // Render the paragraph.
        self.paragraph.render_ref(inner, buf, state);

        // Render the scrollbar.
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("╮"))
            .end_symbol(Some("╯"));

        state.scrollbar = state.scrollbar.content_length(state.max_scroll_pos);
        state.scrollbar = state.scrollbar.position(state.scroll_pos);
        scrollbar.render(
            main.inner(Margin {
                horizontal: 0,
                vertical: 0,
            }),
            buf,
            &mut state.scrollbar,
        );

        // Highlight the selected search matches.
        let style = if state.search_active {
            theme.highlight.inactive
        } else {
            theme.highlight.active
        };
        if let Some(selected_match) = state.selected_match() {
            let x = selected_match.1 + 2;
            let y = (selected_match.0 + 1).saturating_sub(state.scroll_pos as u16);
            if y > 0 && y < area.height - 1 {
                let area = Rect::new(x, y, state.search.len() as u16, 1);
                Block::new().style(style).render(area, buf);
            }
        }

        // Render the search bar
        let style = if state.search_active {
            theme.search.active
        } else {
            theme.search.inactive
        };

        let mut spans = vec![
            Span::styled(" Search (/): ", style),
            Span::styled(state.search.clone(), style),
        ];
        if state.search_active {
            spans.push(Span::styled(" ", style.reversed()));
        }

        Line::from(spans).render(search, buf);
    }
}

pub(crate) struct Content {}

impl EventfulWidget<AppState, Event> for Content {
    fn key() -> String {
        String::from("ManPageParagraph")
    }

    fn handle_events(_: &EventCtrlRc, ctx: &mut AppState, event: &Event, area: Option<Rect>) {
        let ActiveState::Man(state) = &mut ctx.active_state else {
            return;
        };

        if let Event::Mouse(e) = event {
            let position = Position::new(e.column, e.row);
            let Some(area) = area else {
                return;
            };

            match e.kind {
                MouseEventKind::ScrollUp => state.scroll_up(),
                MouseEventKind::ScrollDown => state.scroll_down(),
                _ => {}
            }

            // if position.y >= area.bottom() {
            //     state.scroll_pos = min(state.scroll_pos + 1, state.max_scroll_pos);
            // }
            // if position.y < area.y {
            //     state.scroll_pos = state.scroll_pos.saturating_sub(1);
            // }
            //
            // if let
            // log_to_file(format!("{:} {}", position.y, area.bottom()));

            if area.contains(position) {
                // log_to_file(format!("MOUSE EVENT {area:?}"));
            } else {
                // log_to_file(format!("MOUSE EVENT MISSED"));
            }
        }
    }
}

impl StatefulWidgetRef for Content {
    type State = ManPageState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();

        // Render the paragraph.
        state.max_scroll_pos = state.text.height().saturating_sub(area.height as usize);
        state.page_height = area.height as usize;

        let style = if state.search_active {
            theme.block.inactive
        } else {
            theme.block.active
        };

        let mut lines: Vec<Line> = Vec::new();
        for line in state.text.lines.clone() {
            let mut new_line: Vec<Span> = Vec::new();
            for span in line {
                if state.search_active {
                    new_line.push(span.style(style));
                } else {
                    new_line.push(span);
                }
            }
            lines.push(Line::from(new_line));
        }

        Paragraph::new(lines)
            .scroll((state.scroll_pos as u16, 0))
            .render(area, buf);
    }
}

fn find_matches_positions(input: &Text, query: &str) -> Vec<(u16, u16)> {
    let mut positions = Vec::new();

    for (current_row, line) in input.lines.clone().into_iter().enumerate() {
        let line = line.to_string();
        for (index, _) in line.to_lowercase().match_indices(&query.to_lowercase()) {
            positions.push((current_row as u16, index as u16));
        }
    }

    positions
}
