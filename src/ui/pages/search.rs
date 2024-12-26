use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::app::AppContext;
use crate::ui::events::EventHandler;

#[derive(Default)]
pub(crate) struct SearchPage {}

#[derive(Default)]
pub(crate) struct SearchPageState {}

impl SearchPageState {
    pub(crate) fn on_mount(ctx: &mut AppContext, events: &mut EventHandler) {}
}

impl SearchPageState {
    pub(crate) fn new(ctx: &mut AppContext) -> Self {
        Self {}
    }

    pub(crate) fn on_drop(ctx: &mut AppContext) {}
}

impl StatefulWidget for SearchPage {
    type State = SearchPageState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Paragraph::new("x").render(area, buf);
    }
}
