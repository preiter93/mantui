use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Widget};
use tachyonfx::CenteredShrink;
use throbber_widgets_tui::{Throbber, ThrobberState};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::ui::app::AppContext;
use crate::ui::theme::get_theme;

use super::{ManPageState, Page, drop_page};

#[derive(Default)]
pub(crate) struct ListPage {}

#[derive(Default)]
pub(crate) struct ListPageState {
    pub(crate) commands: Option<Vec<String>>,
    list: ListState,
    num_elements: u16,
    search_active: bool,
    search: String,
    page_width: usize,
    throbber: ThrobberState,
}

impl ListPageState {
    pub(crate) fn new(ctx: &mut AppContext) -> Self {
        Self::on_mount(ctx);

        let mut list = ListState::default();
        list.select(ctx.selected_index);

        Self {
            commands: ctx.commands.clone(),
            list,
            num_elements: 0,
            search_active: false,
            search: ctx.search.clone(),
            page_width: 0,
            throbber: ThrobberState::default(),
        }
    }

    fn filtered_commands(&self) -> Option<Vec<String>> {
        let Some(commands) = &self.commands else {
            return None;
        };

        let filtered = commands
            .iter()
            .filter(|x| x.to_lowercase().contains(&self.search.to_lowercase()))
            .cloned()
            .collect();

        Some(filtered)
    }

    fn selected_commands(&self) -> Option<String> {
        let Some(commands) = &self.filtered_commands() else {
            return None;
        };

        self.list.selected.map(|i| commands[i].clone())
    }

    pub(crate) fn on_mount(ctx: &mut AppContext) {
        ctx.notifier.listen("list", |(ctx, key)| {
            let Page::List(state) = &mut ctx.current_page else {
                return;
            };

            match key.code {
                KeyCode::Char('j') if !state.search_active => {
                    state.list.next();
                }
                KeyCode::Char('k') if !state.search_active => {
                    state.list.previous();
                }
                KeyCode::Char('d')
                    if key.modifiers == KeyModifiers::CONTROL && !state.search_active =>
                {
                    for _ in 0..state.num_elements / 2 {
                        state.list.next();
                    }
                }
                KeyCode::Char('u')
                    if key.modifiers == KeyModifiers::CONTROL && !state.search_active =>
                {
                    for _ in 0..state.num_elements / 2 {
                        state.list.previous();
                    }
                }
                KeyCode::Enter if !state.search_active => {
                    if let Some(command) = state.selected_commands() {
                        ctx.search = state.search.clone();
                        ctx.selected_index = state.list.selected;

                        let width = state.page_width;
                        drop_page(ctx);
                        ctx.current_page = Page::Desc(ManPageState::new(ctx, &command, width));
                    }
                }
                KeyCode::Char('/') if !state.search_active => {
                    state.search_active = true;
                    state.list.selected = None;
                }
                KeyCode::Esc | KeyCode::Enter if state.search_active => {
                    state.search_active = false;
                }
                KeyCode::Backspace if state.search_active => {
                    state.search.pop();
                }
                KeyCode::Char(ch) if state.search_active => {
                    state.search.push(ch);
                }
                _ => {}
            }
        });
    }

    pub(crate) fn on_drop(ctx: &mut AppContext) {
        ctx.notifier.unlisten("home");
    }
}

impl StatefulWidget for ListPage {
    type State = ListPageState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();

        let [list, search] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .areas(area);

        let block_style = if state.search_active {
            theme.block.inactive
        } else {
            theme.block.active
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(block_style);
        let inner = block.inner(list);
        block.render(list, buf);

        let command_list = CommandList;
        command_list.render(inner, buf, state);

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

        state.page_width = inner.width as usize;
    }
}

struct CommandList;

impl StatefulWidget for CommandList {
    type State = ListPageState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();

        state.num_elements = area.height;

        let commands = state.filtered_commands();
        let Some(commands) = commands else {
            let area = area.inner_centered(10, 2);
            let [throbber, text] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1)])
                .areas(area);
            StatefulWidget::render(
                Throbber::default().style(theme.base),
                throbber.inner_centered(1, 1),
                buf,
                &mut state.throbber,
            );
            Line::from("Loading...")
                .style(theme.base)
                .italic()
                .render(text, buf);
            state.throbber.calc_next();
            return;
        };

        let builder = ListBuilder::new(|context| {
            let command = commands[context.index].clone();

            let mut line = Line::from(command);

            let style = if state.search_active {
                theme.list.inactive
            } else if context.index % 2 == 0 {
                theme.list.even
            } else {
                theme.list.odd
            };
            line = line.style(style);

            if context.is_selected {
                line = line.style(theme.list.selected);
            }

            (line, 1)
        });

        ListView::new(builder, commands.len())
            .scroll_padding(2)
            .infinite_scrolling(false)
            .render(area, buf, &mut state.list);
    }
}
