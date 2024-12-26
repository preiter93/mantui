use std::{
    cell::OnceCell,
    sync::{Arc, Mutex, OnceLock},
};

use ratatui::style::{Color, Style};

#[must_use]
pub(super) fn get_theme() -> &'static Theme {
    THEME.get_or_init(Theme::default)
}

static THEME: OnceLock<Theme> = OnceLock::new();

#[derive(Debug, Clone)]
pub(super) struct Theme {
    pub(super) base: Style,
}

impl Default for Theme {
    fn default() -> Self {
        let default_style = StyleProperties {
            foreground: Some(Color::White),
            background: None,
        };

        Self {
            base: default_style.into(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(super) struct StyleProperties {
    foreground: Option<Color>,
    background: Option<Color>,
}

impl From<StyleProperties> for Style {
    fn from(value: StyleProperties) -> Self {
        let mut style = Self::default();

        if let Some(fg) = value.foreground {
            style = style.fg(fg);
        }

        if let Some(bg) = value.background {
            style = style.bg(bg);
        }

        style
    }
}
