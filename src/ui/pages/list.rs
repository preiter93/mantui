use std::cmp::min;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Widget};
use tachyonfx::CenteredShrink;
use throbber_widgets_tui::{Throbber, ThrobberState};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::ui::app::{AppContext, load_commands_in_background};
use crate::ui::theme::get_theme;

use super::{ManPageState, Page, drop_page};

#[derive(Default)]
pub(crate) struct ListPage {}

#[derive(Default)]
pub(crate) struct ListPageState {
    pub(crate) commands: Option<Vec<String>>,
    command_list: ListState,
    section_list: ListState,
    num_elements: u16,
    search_active: bool,
    search: String,
    page_width: usize,
    throbber: ThrobberState,
    section_active: bool,
}

macro_rules! select_section {
    ($state:expr, $ctx:expr, $section:expr) => {
        if $state.section_list.selected != Some($section) {
            $state.commands = None;
            $state.section_list.select(Some($section));
            $state.command_list.select(None);
            $ctx.selected_section = $section;
            load_commands_in_background($ctx, $section);
        }
    };
}

impl ListPageState {
    pub(crate) fn new(ctx: &mut AppContext) -> Self {
        Self::on_mount(ctx);

        let mut command_list = ListState::default();
        command_list.select(ctx.selected_command);

        let mut section_list = ListState::default();
        section_list.select(Some(ctx.selected_section));

        Self {
            commands: ctx.commands.clone(),
            command_list,
            section_list,
            num_elements: 0,
            search_active: false,
            section_active: false,
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

        self.command_list.selected.map(|i| commands[i].clone())
    }

    pub(crate) fn on_mount(ctx: &mut AppContext) {
        ctx.notifier.listen("list", |(ctx, key)| {
            let Page::List(state) = &mut ctx.current_page else {
                return;
            };

            match key.code {
                KeyCode::Char('j') if !state.search_active => {
                    if state.section_active {
                        let s = min(state.section_list.selected.unwrap() + 1, 8);
                        select_section!(state, ctx, s);
                    } else {
                        state.command_list.next();
                    }
                }
                KeyCode::Char('k') if !state.search_active => {
                    if state.section_active {
                        let s = state.section_list.selected.unwrap().saturating_sub(1);
                        select_section!(state, ctx, s);
                    } else {
                        state.command_list.previous();
                    }
                }
                // KeyCode::Char('h') | KeyCode::Char('l') if !state.search_active => {
                //     state.section_active = !state.section_active;
                // }
                KeyCode::Char('d')
                    if key.modifiers == KeyModifiers::CONTROL && !state.search_active =>
                {
                    for _ in 0..state.num_elements / 2 {
                        state.command_list.next();
                    }
                }
                KeyCode::Char('u')
                    if key.modifiers == KeyModifiers::CONTROL && !state.search_active =>
                {
                    for _ in 0..state.num_elements / 2 {
                        state.command_list.previous();
                    }
                }
                KeyCode::Char('1') if !state.search_active => {
                    select_section!(state, ctx, 0);
                }
                KeyCode::Char('2') if !state.search_active => {
                    select_section!(state, ctx, 1);
                }
                KeyCode::Char('3') if !state.search_active => {
                    select_section!(state, ctx, 2);
                }
                KeyCode::Char('4') if !state.search_active => {
                    select_section!(state, ctx, 3);
                }
                KeyCode::Char('5') if !state.search_active => {
                    select_section!(state, ctx, 4);
                }
                KeyCode::Char('6') if !state.search_active => {
                    select_section!(state, ctx, 5);
                }
                KeyCode::Char('7') if !state.search_active => {
                    select_section!(state, ctx, 6);
                }
                KeyCode::Char('8') if !state.search_active => {
                    select_section!(state, ctx, 7);
                }
                KeyCode::Char('9') if !state.search_active => {
                    select_section!(state, ctx, 8);
                }
                KeyCode::Enter if !state.search_active => {
                    if state.section_active {
                        state.section_active = false;
                        return;
                    }

                    if let Some(command) = state.selected_commands() {
                        ctx.search = state.search.clone();
                        ctx.selected_command = state.command_list.selected;

                        let width = state.page_width;
                        drop_page(ctx);
                        ctx.current_page = Page::Desc(ManPageState::new(ctx, &command, width));
                    }
                }
                KeyCode::Char('/') if !state.search_active => {
                    state.search_active = true;
                    state.command_list.selected = None;
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
        state.page_width = area.width as usize;

        let [main, search] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .areas(area);
        let [commands, sections] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(27)])
            .areas(main);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded);

        let command_block_style = if state.search_active || state.section_active {
            theme.block.inactive
        } else {
            theme.block.active
        };
        let command_list = CommandList {
            block: block.clone().style(command_block_style),
        };
        command_list.render(commands, buf, state);

        let section_block_style = if state.section_active {
            theme.block.active
        } else {
            theme.block.inactive
        };

        let section_list = SectionList {
            block: Some(block.style(section_block_style)),
        };
        section_list.render(
            sections.inner(Margin {
                horizontal: 0,
                vertical: 0,
            }),
            buf,
            state,
        );

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

struct CommandList<'a> {
    block: Block<'a>,
}

impl StatefulWidget for CommandList<'_> {
    type State = ListPageState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();

        let area = {
            let inner = self.block.inner(area);
            self.block.render(area, buf);
            inner
        };

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
            .render(area, buf, &mut state.command_list);
    }
}

struct SectionList<'a> {
    block: Option<Block<'a>>,
}

impl StatefulWidget for SectionList<'_> {
    type State = ListPageState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();

        let mut area = area;
        if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            area = inner;
        }

        let sections = [
            "(1) User commands",
            "(2) System calls",
            "(3) Library calls",
            "(4) Special files",
            "(5) File formats",
            "(6) Games",
            "(7) Miscellaneous",
            "(8) System management",
            "(9) Kernel routines",
        ];

        let builder = ListBuilder::new(|context| {
            let section = sections[context.index];

            let mut line = Line::from(section);

            if context.is_selected {
                line = line.style(theme.list.even);
            } else {
                line = line.style(theme.list.inactive);
            }

            (line, 1)
        });

        ListView::new(builder, sections.len())
            .infinite_scrolling(false)
            .render(area, buf, &mut state.section_list);
    }
}
