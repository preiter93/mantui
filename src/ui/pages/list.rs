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

use super::desc::DescPageState;
use super::{Page, drop_page};

#[derive(Default)]
pub(crate) struct ListPage {}

#[derive(Default)]
pub(crate) struct ListPageState {
    commands: Vec<String>,
    list: ListState,
    num_elements: u16,
    search_active: bool,
    search: String,
}

impl ListPageState {
    pub(crate) fn new(ctx: &mut AppContext, commands: Vec<String>) -> Self {
        Self::on_mount(ctx);

        Self {
            commands,
            list: ListState::default(),
            num_elements: 0,
            search_active: false,
            search: String::new(),
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
        if let Some(selected) = self.list.selected {
            Some(self.filtered_commands()[selected].clone())
        } else {
            None
        }
    }

    pub(crate) fn on_mount(ctx: &mut AppContext) {
        let register = &mut ctx.register;
        register.register_event(KeyEvent::from(KeyCode::Char('j')), |(ctx, _)| {
            if let Page::List(state) = &mut ctx.current_page {
                if state.search_active {
                    return;
                }

                state.list.next();
            }
        });

        register.register_event(KeyEvent::from(KeyCode::Char('k')), |(ctx, _)| {
            if let Page::List(state) = &mut ctx.current_page {
                if state.search_active {
                    return;
                }

                state.list.previous();
            }
        });

        register.register_event(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
            |(ctx, _)| {
                if let Page::List(state) = &mut ctx.current_page {
                    if state.search_active {
                        return;
                    }

                    for _ in 0..state.num_elements / 2 {
                        state.list.next();
                    }
                }
            },
        );

        register.register_event(
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            |(ctx, _)| {
                if let Page::List(state) = &mut ctx.current_page {
                    if state.search_active {
                        return;
                    }

                    for _ in 0..state.num_elements / 2 {
                        state.list.previous();
                    }
                }
            },
        );

        register.register_event(KeyEvent::from(KeyCode::Char('/')), |(ctx, event)| {
            if let Page::List(state) = &mut ctx.current_page {
                state.search_active = true;
                state.list.selected = None;
            }
        });

        register.register_event(KeyEvent::from(KeyCode::Esc), |(ctx, event)| {
            if let Page::List(state) = &mut ctx.current_page {
                state.search_active = false;
            }
        });

        register.register_event(RegisterEvent::All, |(ctx, event)| {
            if let Page::List(state) = &mut ctx.current_page {
                if !state.search_active {
                    return;
                }

                if let KeyCode::Char(ch) = event.code {
                    if ch == '/' {
                        return;
                    }
                    log_to_file(format!("push {:}", state.search));
                    state.search.push(ch);
                }
            }
        });

        register.register_event(KeyEvent::from(KeyCode::Backspace), |(ctx, event)| {
            if let Page::List(state) = &mut ctx.current_page {
                if !state.search_active {
                    return;
                }

                state.search.pop();
            }
        });

        register.register_event(KeyEvent::from(KeyCode::Enter), |(ctx, event)| {
            if let Page::List(state) = &mut ctx.current_page {
                if state.search_active {
                    state.search_active = false;
                    return;
                }

                if let Some(command) = state.selected_commands() {
                    drop_page(ctx);
                    ctx.current_page = Page::Desc(DescPageState::new(ctx, &command));
                }
            }
        });
    }

    pub(crate) fn on_drop(ctx: &mut AppContext) {
        let register = &mut ctx.register;
        register.unregister_event(KeyEvent::from(KeyCode::Char('j')));
        register.unregister_event(KeyEvent::from(KeyCode::Char('k')));
        register.unregister_event(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL));
        register.unregister_event(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        register.unregister_event(KeyEvent::from(KeyCode::Char('/')));
        register.unregister_event(KeyEvent::from(KeyCode::Enter));
        register.unregister_event(KeyEvent::from(KeyCode::Esc));
        register.unregister_event(KeyEvent::from(KeyCode::Backspace));
        register.unregister_event(RegisterEvent::All);
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

        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(list);
        block.render(list, buf);

        let command_list = CommandList;
        command_list.render(inner, buf, state);

        if show_search {
            Line::from(format!("/{}", state.search.clone()))
                .style(theme.base)
                .render(search, buf);
        }
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
