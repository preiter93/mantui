use crate::{
    core::get_man_page,
    ui::{app::AppContext, debug::log_to_file, events::EventHandler, theme::get_theme},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use style::Styled;
use tui_widget_list::{ListBuilder, ListState, ListView};

use super::Page;

#[derive(Default)]
pub(crate) struct DescPage {}

#[derive(Default)]
pub(crate) struct DescPageState {
    description: String,
    scroll_pos: u16,
}

impl DescPageState {
    pub(crate) fn on_mount(ctx: &mut AppContext) {
        ctx.notifier.listen("desc", |(ctx, event)| {
            let Page::Desc(state) = &mut ctx.current_page else {
                return;
            };

            match event.code {
                KeyCode::Char('j') => {
                    state.scroll_pos += 1;
                }
                KeyCode::Char('j') => {}
                _ => {
                    state.scroll_pos = state.scroll_pos.saturating_sub(1);
                }
            }
        });
    }

    pub(crate) fn on_drop(ctx: &mut AppContext) {
        ctx.notifier.unlisten("desc");
    }
}

impl DescPageState {
    pub(crate) fn new(ctx: &mut AppContext, command: &str) -> Self {
        let description = get_man_page(command).unwrap_or_default();

        Self::on_mount(ctx);

        Self {
            description,
            // list: ListState::default(),
            scroll_pos: 0,
        }
    }
}

impl StatefulWidget for DescPage {
    type State = DescPageState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();
        let block = Block::default().style(theme.base).borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, buf);

        // let lines = state.description.lines().collect::<Vec<_>>();
        // let builder = ListBuilder::new(|ctx| {
        //     let mut line = Line::from(lines[ctx.index]);
        //
        //     if ctx.is_selected {
        //         line = line.style(theme.list.selected);
        //     }
        //
        //     (line, 1)
        // });
        // log_to_file(format!("{}", lines[0]));
        // log_to_file(format!("{}", lines[1]));
        // log_to_file(format!("{}", lines[2]));
        // log_to_file(format!("{}", lines[3]));
        //
        // ListView::new(builder, lines.len())
        //     .style(theme.base)
        //     .render(inner, buf, &mut state.list);
        Paragraph::new(state.description.clone())
            .wrap(Wrap { trim: true })
            .scroll((state.scroll_pos, 0))
            .render(inner, buf);
    }
}
