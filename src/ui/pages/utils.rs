#![allow(clippy::cast_possible_truncation)]
use std::cmp::{Ordering, max, min};

use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    text::Text,
};

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

pub(super) fn find_matches(text: &Text, query: &str) -> Vec<(u16, u16)> {
    let mut positions = Vec::new();

    for (current_row, line) in text.lines.clone().into_iter().enumerate() {
        let line = line.to_string();
        for (index, _) in line.to_lowercase().match_indices(&query.to_lowercase()) {
            positions.push((current_row as u16, index as u16));
        }
    }

    positions
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct PositionAbsolut(Position);

impl PositionAbsolut {
    pub(super) fn new(x: u16, y: u16) -> Self {
        Self(Position::new(x, y))
    }

    pub(super) fn from_screen(
        position: PositionScreen,
        scroll_offset: u16,
        padding_x: u16,
        padding_y: u16,
    ) -> Self {
        Self::new(
            position.0.x.saturating_sub(padding_x),
            (position.0.y + scroll_offset).saturating_sub(padding_y),
        )
    }

    pub(super) fn into_screen(
        self,
        scroll_offset: u16,
        padding_x: u16,
        padding_y: u16,
    ) -> PositionScreen {
        PositionScreen::new(
            self.0.x + padding_x,
            (self.0.y + padding_y).saturating_sub(scroll_offset),
        )
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct PositionScreen(Position);

impl PositionScreen {
    pub(super) fn new(x: u16, y: u16) -> Self {
        Self(Position::new(x, y))
    }
}

#[derive(Debug, Clone)]
pub(super) struct Selection {
    pub(super) start: PositionAbsolut,
    pub(super) end: PositionAbsolut,
}

impl Selection {
    pub(super) fn new(start: PositionAbsolut, end: PositionAbsolut) -> Self {
        Self { start, end }
    }

    fn start(&self) -> PositionAbsolut {
        if self.is_reversed() {
            self.end
        } else {
            self.start
        }
    }

    fn end(&self) -> PositionAbsolut {
        if self.is_reversed() {
            self.start
        } else {
            self.end
        }
    }

    fn is_reversed(&self) -> bool {
        match self.end.0.y.cmp(&self.start.0.y) {
            Ordering::Less => true,
            Ordering::Equal => self.end.0.x < self.start.0.x,
            Ordering::Greater => false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn iter_on_screen(
        &self,
        min_x: u16,
        max_x: u16,
        min_y: u16,
        max_y: u16,
        offset_y: u16,
        padding_x: u16,
        padding_y: u16,
    ) -> Option<SelectionIterator> {
        let mut start = self.start().into_screen(offset_y, padding_x, padding_y).0;
        let mut end = self.end().into_screen(offset_y, padding_x, padding_y).0;

        if end.y < min_y {
            return None;
        }

        start.y = max(start.y, min_y);
        start.y = min(start.y, max_y);
        end.y = min(end.y, max_y);

        Some(SelectionIterator::new(start, end, min_x, max_x))
    }
}

pub(super) struct SelectionIterator {
    current: Position,
    end: Position,
    min_x: u16,
    max_x: u16,
    done: bool,
}

impl SelectionIterator {
    fn new(start: Position, end: Position, min_x: u16, max_x: u16) -> Self {
        Self {
            current: start,
            end,
            min_x,
            max_x,
            done: false,
        }
    }
}

impl Iterator for SelectionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let current_position = self.current;

        if is_after_or_equal(self.current, self.end) {
            self.done = true;
            return Some(current_position);
        }

        self.current.x += 1;
        if self.current.x >= self.max_x {
            self.current.x = self.min_x;
            self.current.y += 1;
        }

        Some(current_position)
    }
}

fn is_after_or_equal(current: Position, other: Position) -> bool {
    current.y > other.y || (current.y == other.y && current.x >= other.x)
}

pub(super) fn text_to_lines(text: &Text) -> Vec<String> {
    text.lines
        .iter()
        .map(|line| line.spans.iter().map(|x| x.content.to_string()).collect())
        .collect()
}

pub(super) fn extract_text_from_lines(lines: &[String], selection: &Selection) -> String {
    let start_row = selection.start().0.y as usize;
    let start_col = selection.start().0.x as usize;
    let end_row = selection.end().0.y as usize;
    let end_col = selection.end().0.x as usize;

    if start_row > end_row || (start_row == end_row && start_col > end_col) {
        return String::new();
    }

    let mut extracted_text = String::new();

    for (row_idx, line) in lines.iter().enumerate() {
        if row_idx < start_row || row_idx > end_row {
            continue;
        }

        if row_idx == start_row && row_idx == end_row {
            // Single-line selection
            let end_col = end_col.min(line.len().saturating_sub(1));
            if let Some(slice) = line.get(start_col..=end_col) {
                extracted_text.push_str(slice);
            }
        } else if row_idx == start_row {
            // Slice from start_col to the end of the line
            if let Some(slice) = line.get(start_col..) {
                extracted_text.push_str(slice);
                extracted_text.push('\n');
            }
        } else if row_idx == end_row {
            // Ending row: slice from the beginning to end_col
            let end_col = end_col.min(line.len().saturating_sub(1));
            if let Some(slice) = line.get(..=end_col) {
                extracted_text.push_str(slice);
            }
        } else {
            // Entire row is within the selection range
            extracted_text.push_str(line);
            extracted_text.push('\n');
        }
    }

    extracted_text
}
