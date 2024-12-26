use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(super) fn centered_rect(area: Rect, widget_height: u16) -> Rect {
    if area.height <= widget_height {
        return area;
    }
    let padding = area.height - widget_height;

    let [_, area, _] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(padding / 2),
            Constraint::Length(widget_height),
            Constraint::Length(padding / 2 + padding % 2),
        ])
        .areas(area);

    area
}
