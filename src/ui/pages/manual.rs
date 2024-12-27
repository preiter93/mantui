#![allow(clippy::cast_possible_truncation)]
use std::{cmp::min, time::Duration};

use crate::{
    core::get_manual,
    ui::{
        app::{AppContext, poll_commands},
        debug::log_to_file,
        events::EventHandler,
        theme::get_theme,
    },
};
use ansi_to_tui::IntoText;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::*,
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
};
use style::Styled;
use text::ToText;
use tui_widget_list::{ListBuilder, ListState, ListView};

use super::{ListPageState, Page, drop_page};

#[derive(Default)]
pub(crate) struct ManPage {}

#[derive(Default)]
pub(crate) struct ManPageState {
    command: String,
    manual: String,
    text: Text<'static>,
    scroll_pos: usize,
    page_height: usize,
    max_scroll_pos: usize,
    scrollbar: ScrollbarState,
}

impl ManPageState {
    pub(crate) fn on_mount(ctx: &mut AppContext) {
        ctx.notifier.listen("desc", |(ctx, event)| {
            let Page::Desc(state) = &mut ctx.current_page else {
                return;
            };

            match event.code {
                KeyCode::Char('j') => {
                    state.scroll_pos = min(state.scroll_pos + 1, state.max_scroll_pos);
                }
                KeyCode::Char('k') => {
                    state.scroll_pos = state.scroll_pos.saturating_sub(1);
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
                KeyCode::Esc => {
                    let commands = poll_commands(Duration::from_millis(1000));
                    drop_page(ctx);
                    ctx.current_page = Page::List(ListPageState::new(ctx, commands));
                }
                _ => {}
            }
        });
    }

    pub(crate) fn on_drop(ctx: &mut AppContext) {
        ctx.notifier.unlisten("desc");
    }
}

impl ManPageState {
    pub(crate) fn new(ctx: &mut AppContext, command: &str, width: usize) -> Self {
        Self::on_mount(ctx);
        let reduced_width = (width as f64 * 0.9) as u16;

        let width = format!("{reduced_width}");
        log_to_file(&width);
        let manual = get_manual(command, &width).unwrap_or_default();

        let text =
            IntoText::into_text(&manual).unwrap_or(Text::from("Could not convert ansi to tui."));

        let scrollbar = ScrollbarState::new(0).position(0);

        Self {
            command: command.to_string(),
            manual,
            scroll_pos: 0,
            page_height: 0,
            max_scroll_pos: 0,
            text,
            scrollbar,
        }
    }
}

impl StatefulWidget for ManPage {
    type State = ManPageState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = get_theme();
        let block = Block::default()
            .style(theme.base)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .padding(Padding::horizontal(1));
        let inner = block.inner(area);
        block.render(area, buf);

        state.max_scroll_pos = state.text.height().saturating_sub(inner.height as usize);
        state.page_height = area.height as usize;

        Paragraph::new(state.text.clone())
            .scroll((state.scroll_pos as u16, 0))
            .render(inner, buf);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("╮"))
            .end_symbol(Some("╯"));

        state.scrollbar = state.scrollbar.content_length(state.max_scroll_pos);
        state.scrollbar = state.scrollbar.position(state.scroll_pos);
        scrollbar.render(
            area.inner(Margin {
                horizontal: 0,
                vertical: 0,
            }),
            buf,
            &mut state.scrollbar,
        );
    }
}
