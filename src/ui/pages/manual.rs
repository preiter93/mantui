#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
use std::cmp::{max, min};

use crate::{
    core::get_manual,
    ui::{
        app::{ActivePage, ActiveState, AppState},
        events::{Event, EventContext, EventController, EventfulWidget, IStatefulWidget},
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
    content: IStatefulWidget<Content>,
    search: IStatefulWidget<Search>,
}

impl ManPage {
    pub(crate) fn new(controller: &EventController) -> Self {
        Self {
            content: IStatefulWidget::new(Content, controller),
            search: IStatefulWidget::new(Search, controller),
        }
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
    selection: Option<MouseSelection>,
    selection_active: bool,
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
            selection: None,
            selection_active: false,
        }
    }
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

impl EventfulWidget<AppState, Event> for ManPage {
    fn unique_key() -> String {
        String::from("ManPage")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, _: Option<Rect>) {
        let ActiveState::Man(page_state) = &mut state.active_state else {
            return;
        };

        if let Event::Key(event) = ctx.event {
            match event.code {
                KeyCode::Char(ch)
                    if page_state.search_active && event.modifiers != KeyModifiers::CONTROL =>
                {
                    page_state.selected_match = None;
                    page_state.search.push(ch);
                    page_state.matches =
                        find_matches_positions(&page_state.text, &page_state.search);
                    page_state.select_next_search();
                }
                KeyCode::Char('j') => {
                    page_state.scroll_down();
                }
                KeyCode::Char('k') => {
                    page_state.scroll_up();
                }
                KeyCode::Char('d') if event.modifiers == KeyModifiers::CONTROL => {
                    page_state.scroll_pos = min(
                        page_state.scroll_pos + page_state.page_height / 2,
                        page_state.max_scroll_pos,
                    );
                }
                KeyCode::Char('u') if event.modifiers == KeyModifiers::CONTROL => {
                    page_state.scroll_pos = page_state
                        .scroll_pos
                        .saturating_sub(page_state.page_height / 2);
                }
                KeyCode::Char('G') if event.modifiers == KeyModifiers::SHIFT => {
                    page_state.scroll_pos = page_state.max_scroll_pos;
                }
                KeyCode::Char('g') => {
                    page_state.scroll_pos = 0;
                }

                KeyCode::Char('N') if event.modifiers == KeyModifiers::SHIFT => {
                    page_state.select_previous_search();
                }
                KeyCode::Char('n') => {
                    page_state.select_next_search();
                }
                KeyCode::Char('/') => {
                    page_state.search_active = true;
                }
                KeyCode::Backspace if page_state.search_active => {
                    page_state.search.pop();
                }
                KeyCode::Esc => {
                    if page_state.search_active {
                        page_state.search_active = false;
                    } else if page_state.selection.is_some() {
                        page_state.selection = None;
                    } else if page_state.search.is_empty() {
                        let page_state = ListPageState::new(state);
                        state.active_state = ActiveState::List(page_state);

                        let page = ListPage::new(ctx.controller);
                        let page = IStatefulWidget::new(page, ctx.controller);
                        state.active_page = ActivePage::List(page);
                    } else {
                        page_state.search = String::new();
                        page_state.matches = Vec::new();
                        page_state.selected_match = None;
                    }
                }
                KeyCode::Enter if page_state.search_active => {
                    page_state.search_active = false;
                }
                _ => {}
            }
        };
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

        // Render the content.
        self.content.render_ref(inner, buf, state);

        // Render the search
        self.search.render_ref(search, buf, state);

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
    }
}

pub(crate) struct Content;

impl EventfulWidget<AppState, Event> for Content {
    fn unique_key() -> String {
        String::from("ManPageContent")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, area: Option<Rect>) {
        let ActiveState::Man(page_state) = &mut state.active_state else {
            return;
        };

        if let Event::Mouse(e) = ctx.event {
            let position = Position::new(e.column, e.row);
            let Some(area) = area else {
                return;
            };
            if !area.contains(position) {
                // if let MouseEventKind::Drag(_) = e.kind {
                // if position.y >= area.bottom() {
                //     page_state.scroll_down();
                // } else {
                //     page_state.scroll_up();
                // }
                // }
                return;
            }

            let scroll_pos = page_state.scroll_pos as u16;
            match e.kind {
                MouseEventKind::ScrollUp => page_state.scroll_up(),
                MouseEventKind::ScrollDown => page_state.scroll_down(),
                MouseEventKind::Down(_) => {
                    page_state.search_active = false;
                    page_state.selection_active = false;
                    page_state.selection = Some(MouseSelection::new(
                        offset_position(position, scroll_pos),
                        offset_position(position, scroll_pos),
                    ));
                }
                MouseEventKind::Drag(_) => {
                    page_state.selection_active = true;
                    if let Some(selection) = &page_state.selection {
                        page_state.selection = Some(MouseSelection::new(
                            selection.start,
                            offset_position(position, scroll_pos),
                        ));
                    }
                }
                _ => {}
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

        // Highlight the search matches.
        let padding_x = 2;
        let padding_y = 1;
        let style = if state.search_active {
            theme.highlight.inactive
        } else {
            theme.highlight.active
        };
        if let Some(selected_match) = state.selected_match() {
            let x = selected_match.1 + padding_x;
            let y = (selected_match.0 + padding_y).saturating_sub(state.scroll_pos as u16);
            if y > 0 && y < area.height - 1 {
                let area = Rect::new(x, y, state.search.len() as u16, 1);
                Block::new().style(style).render(area, buf);
            }
        }

        // Highlight the mouse selection.
        if !state.selection_active {
            return;
        }

        if let Some(selection) = state.selection.clone() {
            for position in selection.iter_positions(
                area.left(),
                area.right(),
                area.top(),
                state.scroll_pos as u16,
            ) {
                let Some(cell) = buf.cell_mut(position) else {
                    continue;
                };
                cell.set_style(theme.highlight.active);
            }
        }
    }
}

struct Search;

impl EventfulWidget<AppState, Event> for Search {
    fn unique_key() -> String {
        String::from("ManPageSearch")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, area: Option<Rect>) {
        let ActiveState::Man(page_state) = &mut state.active_state else {
            return;
        };

        if let Event::Mouse(e) = ctx.event {
            let position = Position::new(e.column, e.row);
            let Some(area) = area else {
                return;
            };

            if !area.contains(position) {
                return;
            }

            if let MouseEventKind::Down(_) = e.kind {
                page_state.search_active = true;
                page_state.selection = None;
                page_state.selection_active = false;
            }
        }
    }
}

impl StatefulWidgetRef for Search {
    type State = ManPageState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();

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

        Line::from(spans).render(area, buf);
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

#[derive(Clone)]
struct MouseSelection {
    start: Position,
    end: Position,
}

fn offset_position(pos: Position, scroll_pos: u16) -> Position {
    let mut pos = pos;
    pos.y += scroll_pos;
    pos
}

impl MouseSelection {
    fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    fn start(&self) -> Position {
        if is_greater(self.start, self.end) {
            self.start
        } else {
            self.end
        }
    }

    fn end(&self) -> Position {
        if is_greater(self.start, self.end) {
            self.end
        } else {
            self.start
        }
    }

    /// Returns an iterator that iterates from `start` to `end`, returning positions.
    fn iter_positions(
        &self,
        min_x: u16,
        max_x: u16,
        min_y: u16,
        offset_y: u16,
    ) -> MouseSelectionIterator {
        let mut start = self.start();
        let mut end = self.end();

        start.y = max(start.y.saturating_sub(offset_y), min_y);
        end.y = end.y.saturating_sub(offset_y);

        MouseSelectionIterator::new(start, end, min_x, max_x, min_y)
    }
}

struct MouseSelectionIterator {
    current: Position,
    end: Position,
    min_y: u16,
    min_x: u16,
    max_x: u16,
    done: bool,
}

fn is_greater(start: Position, end: Position) -> bool {
    if end.y > start.y {
        return true;
    } else if end.y == start.y {
        end.x >= start.x
    } else {
        false
    }
}

impl MouseSelectionIterator {
    fn new(start: Position, end: Position, min_x: u16, max_x: u16, min_y: u16) -> Self {
        Self {
            current: start,
            end,
            min_x,
            max_x,
            min_y,
            done: false,
        }
    }
}

impl Iterator for MouseSelectionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if self.end.y < self.min_y {
            self.done;
            return None;
        }

        let current_position = self.current.clone();

        if self.current == self.end {
            self.done = true;
            return Some(current_position);
        }

        self.current.x += 1;
        if self.current.x >= self.max_x {
            self.current.x = self.min_x;
            self.current.y += 1;
        }

        Some(current_position)
    }
}
