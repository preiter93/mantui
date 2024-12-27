use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use style::Styled;
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::core::list_user_commands;
use crate::ui::app::AppContext;
use crate::ui::debug::log_to_file;
use crate::ui::events::{EventHandler, RegisterEvent};
use crate::ui::theme::get_theme;

use super::{ManPageState, Page, drop_page};

#[derive(Default)]
pub(crate) struct ListPage {}

#[derive(Default)]
pub(crate) struct ListPageState {
    commands: Vec<String>,
    list: ListState,
    num_elements: u16,
    search_active: bool,
    search: String,
    page_width: usize,
}

impl ListPageState {
    pub(crate) fn new(ctx: &mut AppContext, commands: Vec<String>) -> Self {
        Self::on_mount(ctx);

        let mut list = ListState::default();
        list.select(ctx.selected_index);

        Self {
            commands,
            list,
            num_elements: 0,
            search_active: false,
            search: ctx.search.clone(),
            page_width: 0,
        }
    }

    fn filtered_commands(&self) -> Vec<String> {
        self.commands
            .iter()
            .filter(|x| x.to_lowercase().contains(&self.search.to_lowercase()))
            .cloned()
            .map(|x| x.to_lowercase())
            .collect()
    }

    fn selected_commands(&self) -> Option<String> {
        self.list
            .selected
            .map(|selected| self.filtered_commands()[selected].clone())
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

        let show_search = state.search_active || !state.search.is_empty();

        let [list, search] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(show_search.into())])
            .areas(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded);
        let inner = block.inner(list);
        block.render(list, buf);

        let command_list = CommandList;
        command_list.render(inner, buf, state);

        if show_search {
            Line::from(format!("/{}", state.search.clone()))
                .style(theme.base)
                .render(search, buf);
        }

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

        let builder = ListBuilder::new(|context| {
            let command = commands[context.index].clone();

            let mut line = Line::from(command);

            if context.index % 2 == 0 {
                line = line.style(theme.list.even);
            } else {
                line = line.style(theme.list.odd);
            }

            if context.is_selected {
                line = line.style(theme.list.selected);
            }

            (line, 1)
        });

        ListView::new(builder, commands.len()).render(area, buf, &mut state.list);
    }
}
