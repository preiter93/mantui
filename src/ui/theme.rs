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
    pub(super) list: ListStyle,
}

impl Default for Theme {
    fn default() -> Self {
        let default_style = StyleProperties {
            foreground: Some(Color::White),
            background: None,
        };

        let orange = Color::Rgb(255, 153, 0);
        let charcoal = Color::Rgb(28, 28, 32);

        Self {
            base: default_style.into(),
            list: ListStyle {
                even: default_style.into(),
                odd: StyleProperties {
                    foreground: Some(Color::White),
                    background: None,
                    // background: Some(charcoal),
                }
                .into(),
                selected: StyleProperties {
                    foreground: Some(charcoal),
                    background: Some(orange),
                }
                .into(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ListStyle {
    pub(super) even: Style,
    pub(super) odd: Style,
    pub(super) selected: Style,
}

#[derive(Debug, Default, Clone, Copy)]
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
