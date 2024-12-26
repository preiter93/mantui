use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::app::AppContext;
use crate::ui::events::EventHandler;

#[derive(Default)]
pub(crate) struct ManPage {}

#[derive(Default)]
pub(crate) struct ManPageState {}

impl ManPageState {
    pub(crate) fn new(ctx: &mut AppContext) -> Self {
        Self {}
    }

    pub(crate) fn on_drop(ctx: &mut AppContext) {}
}

impl StatefulWidget for ManPage {
    type State = ManPageState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Paragraph::new("x").render(area, buf);
    }
}
