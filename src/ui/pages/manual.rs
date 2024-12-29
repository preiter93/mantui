#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss, clippy::cast_precision_loss)]
use std::cmp::min;

use crate::{
    core::get_manual,
    ui::{
        app::{ActivePage, ActiveState, AppState},
        events::{Event, EventContext, EventController, EventfulWidget, IStatefulWidget},
        theme::get_theme,
    },
};
use ansi_to_tui::IntoText;
use arboard::Clipboard;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers, MouseEventKind},
    prelude::*,
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidgetRef,
    },
};

use super::{
    ListPage, ListPageState,
    utils::{
        PositionAbsolut, PositionScreen, Selection, extract_text_from_lines, find_matches,
        text_to_lines,
    },
};

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
    scroll_offset: usize,
    page_height: usize,
    num_lines: usize,
    max_scroll_pos: usize,
    scrollbar: ScrollbarState,
    search_active: bool,
    search: String,
    selected_match: Option<usize>,
    matches: Vec<(u16, u16)>,
    selection: Option<Selection>,
    selection_active: bool,
    clipboard: Option<Clipboard>,
    padding_x: u16,
    padding_y: u16,
}

impl ManPageState {
    pub(crate) fn new(command: &str, width: usize) -> Self {
        let reduced_width = (width as f64 * 0.9) as u16;

        let width = format!("{reduced_width}");
        let text_raw = get_manual(command, &width).unwrap_or_default();

        let text =
            IntoText::into_text(&text_raw).unwrap_or(Text::from("Could not convert ansi to tui."));
        let num_lines = text.lines.len();

        let scrollbar = ScrollbarState::new(0).position(0);

        let clipboard = Clipboard::new().ok();

        Self {
            scroll_offset: 0,
            page_height: 0,
            max_scroll_pos: 0,
            num_lines,
            text,
            scrollbar,
            search: String::new(),
            matches: Vec::new(),
            selected_match: None,
            search_active: false,
            selection: None,
            selection_active: false,
            clipboard,
            padding_x: 2,
            padding_y: 1,
        }
    }
    fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
    fn scroll_down(&mut self) {
        self.scroll_offset = min(self.scroll_offset + 1, self.max_scroll_pos);
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
            let last_visible_row =
                self.scroll_offset + self.page_height.saturating_sub(3) - padding;
            let diff = selected_row.saturating_sub(last_visible_row);
            if diff > 0 {
                self.scroll_offset += diff;
                self.scroll_offset = self.scroll_offset.min(self.max_scroll_pos);
            }

            // Check if the selected match is above the visible range
            let first_visible_row = self.scroll_offset + padding;
            if selected_row < first_visible_row {
                self.scroll_offset = selected_row.saturating_sub(padding);
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
            let last_visible_row =
                self.scroll_offset + self.page_height.saturating_sub(3) - padding;
            let diff = selected_row.saturating_sub(last_visible_row);
            if diff > 0 {
                self.scroll_offset += diff;
                self.scroll_offset = self.scroll_offset.min(self.max_scroll_pos);
            }

            // Check if the selected match is above the visible range
            let first_visible_row = self.scroll_offset + padding;
            if selected_row < first_visible_row {
                self.scroll_offset = selected_row.saturating_sub(padding);
            }
        }
    }

    fn selected_match(&self) -> Option<(u16, u16)> {
        if let Some(index) = self.selected_match {
            return Some(self.matches[index]);
        }

        None
    }

    fn copy_selection(&mut self) {
        if let Some(clipboard) = &mut self.clipboard {
            if let Some(selection) = &self.selection.take() {
                let lines = text_to_lines(&self.text);
                let copied_text = extract_text_from_lines(&lines, selection);
                let _ = clipboard.set_text(copied_text);
            }
        }
    }
}

impl EventfulWidget<AppState, Event> for ManPage {
    fn unique_key() -> String {
        String::from("ManPage")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, area: Option<Rect>) {
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
                    page_state.matches = find_matches(&page_state.text, &page_state.search);
                    page_state.select_next_search();
                }
                KeyCode::Char('j') => {
                    page_state.scroll_down();
                }
                KeyCode::Char('k') => {
                    page_state.scroll_up();
                }
                KeyCode::Char('d') if event.modifiers == KeyModifiers::CONTROL => {
                    page_state.scroll_offset = min(
                        page_state.scroll_offset + page_state.page_height / 2,
                        page_state.max_scroll_pos,
                    );
                }
                KeyCode::Char('u') if event.modifiers == KeyModifiers::CONTROL => {
                    page_state.scroll_offset = page_state
                        .scroll_offset
                        .saturating_sub(page_state.page_height / 2);
                }
                KeyCode::Char('G') if event.modifiers == KeyModifiers::SHIFT => {
                    page_state.scroll_offset = page_state.max_scroll_pos;
                }
                KeyCode::Char('g') => {
                    page_state.scroll_offset = 0;
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
            .padding(Padding::horizontal(state.padding_x.saturating_sub(1)));
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
        state.scrollbar = state.scrollbar.position(state.scroll_offset);
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
                return;
            }

            let scroll_offset = page_state.scroll_offset as u16;
            let click_position = PositionAbsolut::from_screen(
                PositionScreen::new(position.x, position.y),
                scroll_offset,
                page_state.padding_x,
                page_state.padding_y,
            );
            match e.kind {
                MouseEventKind::ScrollUp => page_state.scroll_up(),
                MouseEventKind::ScrollDown => page_state.scroll_down(),
                MouseEventKind::Down(_) => {
                    page_state.search_active = false;
                    page_state.selection_active = false;
                    page_state.selection = Some(Selection::new(click_position, click_position));
                }
                MouseEventKind::Drag(_) => {
                    page_state.selection_active = true;
                    if let Some(selection) = &page_state.selection {
                        page_state.selection =
                            Some(Selection::new(selection.start, click_position));
                    }
                }
                MouseEventKind::Up(_) => {
                    if page_state.selection_active {
                        page_state.copy_selection();
                    }
                    page_state.selection = None;
                    page_state.selection_active = false;
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
            .scroll((state.scroll_offset as u16, 0))
            .render(area, buf);

        // Highlight the search matches.
        let style = if state.search_active {
            theme.highlight.inactive
        } else {
            theme.highlight.active
        };
        if let Some(selected_match) = state.selected_match() {
            let x = selected_match.1 + state.padding_x;
            let y = (selected_match.0 + state.padding_y).saturating_sub(state.scroll_offset as u16);
            if y > 0 && y < area.height - 1 {
                let area = Rect::new(x, y, state.search.len() as u16, 1);
                Block::new().style(style).render(area, buf);
            }
        }

        // Highlight the mouse selection.
        if !state.selection_active {
            return;
        }

        if let Some(iter) = state.selection.clone().and_then(|selection| {
            selection.iter_on_screen(
                area.left(),
                area.right(),
                area.top(),
                min(state.num_lines as u16, area.bottom()),
                state.scroll_offset as u16,
                state.padding_x,
                state.padding_y,
            )
        }) {
            for position in iter {
                let Some(cell) = buf.cell_mut(position) else {
                    continue;
                };
                cell.set_style(theme.highlight.active);
            }
        };
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
