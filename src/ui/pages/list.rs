use std::cmp::min;

use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, StatefulWidgetRef, Widget};
use tachyonfx::CenteredShrink;
use throbber_widgets_tui::{Throbber, ThrobberState};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::ui::app::{load_commands_in_background, ActiveState, AppState, Navigation};
use crate::ui::events::{Event, EventContext, EventController, EventfulWidget, IStatefulWidget};
use crate::ui::theme::get_theme;

macro_rules! select_section {
    ($state:expr, $ctx:expr, $section:expr) => {
        if $state.section_list.selected != Some($section) {
            $state.loaded_commands = None;
            $state.section_list.select(Some($section));
            $state.command_list.select(None);
            $state.search = String::new();
            load_commands_in_background($ctx, $section);
        }
    };
}

pub(crate) struct ListPage {
    commands: IStatefulWidget<Commands>,
    search: IStatefulWidget<Search>,
    section: IStatefulWidget<Section>,
}

impl ListPage {
    pub(crate) fn new(controller: &EventController) -> Self {
        Self {
            commands: IStatefulWidget::new(Commands, controller),
            search: IStatefulWidget::new(Search, controller),
            section: IStatefulWidget::new(Section, controller),
        }
    }
}

impl EventfulWidget<AppState, Event> for ListPage {
    fn unique_key() -> String {
        String::from("ListPage")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, _: Option<Rect>) {
        let ActiveState::List(page_state) = &mut state.active_state else {
            return;
        };

        let Event::Key(key) = ctx.event else {
            return;
        };

        match key.code {
            KeyCode::Char('j') if !page_state.search_active => {
                if page_state.section_active {
                    let s = min(page_state.section_list.selected.unwrap() + 1, 8);
                    select_section!(page_state, state, s);
                } else {
                    page_state.scroll_down();
                }
            }
            KeyCode::Char('k') if !page_state.search_active => {
                if page_state.section_active {
                    let s = page_state.section_list.selected.unwrap().saturating_sub(1);
                    select_section!(page_state, state, s);
                } else {
                    page_state.scroll_up();
                }
            }
            KeyCode::Char('d')
                if key.modifiers == KeyModifiers::CONTROL && !page_state.search_active =>
            {
                for _ in 0..page_state.num_elements / 2 {
                    page_state.command_list.next();
                }
            }
            KeyCode::Char('u')
                if key.modifiers == KeyModifiers::CONTROL && !page_state.search_active =>
            {
                for _ in 0..page_state.num_elements / 2 {
                    page_state.command_list.previous();
                }
            }
            KeyCode::Char('1') if !page_state.search_active => {
                select_section!(page_state, state, 0);
            }
            KeyCode::Char('2') if !page_state.search_active => {
                select_section!(page_state, state, 1);
            }
            KeyCode::Char('3') if !page_state.search_active => {
                select_section!(page_state, state, 2);
            }
            KeyCode::Char('4') if !page_state.search_active => {
                select_section!(page_state, state, 3);
            }
            KeyCode::Char('5') if !page_state.search_active => {
                select_section!(page_state, state, 4);
            }
            KeyCode::Char('6') if !page_state.search_active => {
                select_section!(page_state, state, 5);
            }
            KeyCode::Char('7') if !page_state.search_active => {
                select_section!(page_state, state, 6);
            }
            KeyCode::Char('8') if !page_state.search_active => {
                select_section!(page_state, state, 7);
            }
            KeyCode::Char('9') if !page_state.search_active => {
                select_section!(page_state, state, 8);
            }
            KeyCode::Enter if !page_state.search_active => {
                if page_state.section_active {
                    page_state.section_active = false;
                    return;
                }
                Navigation::navigate_to(&Navigation::Reader, state, ctx.controller);
            }
            KeyCode::Char('/') if !page_state.search_active => {
                page_state.search_active = true;
                page_state.command_list.selected = None;
            }
            KeyCode::Esc | KeyCode::Enter if page_state.search_active => {
                page_state.search_active = false;
            }
            KeyCode::Esc if !page_state.search_active => {
                page_state.search = String::new();
                page_state.command_list.selected = None;
            }
            KeyCode::Backspace if page_state.search_active => {
                page_state.search.pop();
            }
            KeyCode::Char(ch) if page_state.search_active => {
                page_state.command_list.selected = None;
                page_state.search.push(ch);
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub(crate) struct ListPageState {
    loaded_commands: Option<Vec<String>>,
    command_list: ListState,
    section_list: ListState,
    num_elements: u16,
    search_active: bool,
    search: String,
    page_width: usize,
    throbber: ThrobberState,
    section_active: bool,
}

impl ListPageState {
    pub(crate) fn new(state: &mut AppState) -> Self {
        let mut command_list = ListState::default();
        command_list.select(state.selected_command);

        let mut section_list = ListState::default();
        section_list.select(Some(state.selected_section));

        Self {
            loaded_commands: state.loaded_commands.clone(),
            command_list,
            section_list,
            num_elements: 0,
            search_active: false,
            section_active: false,
            search: state.command_search.clone(),
            page_width: 0,
            throbber: ThrobberState::default(),
        }
    }

    fn filtered_commands(&self) -> Option<Vec<String>> {
        let Some(commands) = &self.loaded_commands else {
            return None;
        };

        let filtered = commands
            .iter()
            .filter(|x| x.to_lowercase().contains(&self.search.to_lowercase()))
            .cloned()
            .collect();

        Some(filtered)
    }

    pub(crate) fn loaded_commands(&self) -> Option<Vec<String>> {
        self.loaded_commands.clone()
    }

    pub(crate) fn set_loaded_commands(&mut self, commands: &[String]) {
        self.loaded_commands = Some(commands.to_vec());
    }

    pub(crate) fn page_width(&self) -> usize {
        self.page_width
    }

    pub(crate) fn selected_command(&self) -> Option<String> {
        let Some(commands) = &self.filtered_commands() else {
            return None;
        };

        self.command_list.selected.map(|i| commands[i].clone())
    }

    pub(crate) fn selected_command_index(&self) -> Option<usize> {
        self.command_list.selected
    }

    pub(crate) fn selected_section_index(&self) -> usize {
        self.section_list.selected.unwrap_or_default()
    }

    pub(crate) fn command_search(&self) -> String {
        self.search.to_string()
    }

    fn scroll_down(&mut self) {
        self.command_list.next();
    }

    fn scroll_up(&mut self) {
        self.command_list.previous();
    }
}

impl StatefulWidgetRef for ListPage {
    type State = ListPageState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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
        let command_block = block.clone().style(command_block_style);
        let area_commands = command_block.inner(commands);
        command_block.render(commands, buf);

        let section_block_style = if state.section_active {
            theme.block.active
        } else {
            theme.block.inactive
        };
        let section_block = block.style(section_block_style);
        let area_sections = section_block.inner(sections);
        section_block.render(sections, buf);

        self.commands.render_ref(area_commands, buf, state);

        self.section.render_ref(area_sections, buf, state);

        self.search.render_ref(search, buf, state);
    }
}

struct Commands;

impl EventfulWidget<AppState, Event> for Commands {
    fn unique_key() -> String {
        String::from("ListPageCommands")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, area: Option<Rect>) {
        let ActiveState::List(page_state) = &mut state.active_state else {
            return;
        };

        if let Event::Mouse(e) = ctx.event {
            let Some(area) = area else {
                return;
            };

            let position = Position::new(e.column, e.row);
            if !area.contains(position) {
                return;
            }

            match e.kind {
                MouseEventKind::ScrollUp => page_state.scroll_up(),
                MouseEventKind::ScrollDown => page_state.scroll_down(),
                MouseEventKind::Down(_) => {
                    page_state.search_active = false;

                    let diff = position.y as usize - area.y as usize;
                    let scroll_offset_index = page_state.command_list.scroll_offset_index();

                    let mouse_select = scroll_offset_index + diff;
                    if mouse_select < page_state.filtered_commands().map_or(0, |l| l.len()) {
                        if page_state.command_list.selected == Some(mouse_select) {
                            Navigation::navigate_to(&Navigation::Reader, state, ctx.controller);
                        } else {
                            page_state.command_list.select(Some(mouse_select));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl StatefulWidgetRef for Commands {
    type State = ListPageState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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
            } else {
                theme.list.active
            };
            line = line.style(style);

            if context.is_selected {
                line = line.style(theme.list.selected);
            }

            (line, 1)
        });

        ListView::new(builder, commands.len())
            .infinite_scrolling(false)
            .render(area, buf, &mut state.command_list);
    }
}

struct Section;

impl EventfulWidget<AppState, Event> for Section {
    fn unique_key() -> String {
        String::from("ListPageSection")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, area: Option<Rect>) {
        let ActiveState::List(page_state) = &mut state.active_state else {
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
                let diff = position.y as usize - area.y as usize;
                if diff < 9 {
                    select_section!(page_state, state, diff);
                }
            }
        }
    }
}

impl StatefulWidgetRef for Section {
    type State = ListPageState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();

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
                line = line.style(theme.list.active);
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

struct Search;

impl EventfulWidget<AppState, Event> for Search {
    fn unique_key() -> String {
        String::from("ListPageSearch")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, area: Option<Rect>) {
        let ActiveState::List(page_state) = &mut state.active_state else {
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
            }
        }
    }
}

impl StatefulWidgetRef for Search {
    type State = ListPageState;
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
