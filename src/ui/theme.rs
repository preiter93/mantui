#![allow(unused)]
use std::sync::OnceLock;
use tachyonfx::HslConvertable;

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
    pub(super) search: SearchStyle,
    pub(super) block: BlockStyle,
    pub(super) highlight: HighlightStyle,
}

impl Default for Theme {
    fn default() -> Self {
        let default_style = StyleProperties {
            foreground: Some(Color::White),
            background: None,
        };

        let black = Color::Black;
        let orange = Color::Rgb(255, 153, 0);
        let charcoal = Color::Rgb(28, 28, 32);
        let gray300 = Color::from_hsl(240., 23., 80.);
        let gray500 = Color::from_hsl(240., 10., 30.);
        let gray700 = Color::from_hsl(240., 7., 12.);

        let red500 = Color::Red;
        let red700 = Color::from_hsl(0.0, 70., 38.);

        let inactive = StyleProperties {
            foreground: Some(gray500),
            background: default_style.background,
        }
        .into();

        Self {
            base: default_style.into(),
            list: ListStyle {
                even: default_style.into(),
                odd: StyleProperties {
                    foreground: default_style.foreground,
                    background: None,
                }
                .into(),
                selected: StyleProperties {
                    foreground: Some(charcoal),
                    background: Some(red500),
                }
                .into(),
                inactive,
            },
            search: SearchStyle {
                active: StyleProperties {
                    foreground: default_style.foreground,
                    background: default_style.background,
                }
                .into(),
                inactive,
            },
            highlight: HighlightStyle {
                active: StyleProperties {
                    foreground: Some(black),
                    background: Some(red500),
                }
                .into(),
                inactive: StyleProperties {
                    foreground: Some(black),
                    background: Some(red700),
                }
                .into(),
            },
            block: BlockStyle {
                active: default_style.into(),
                inactive,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ListStyle {
    pub(super) even: Style,
    pub(super) odd: Style,
    pub(super) selected: Style,
    pub(super) inactive: Style,
}

#[derive(Debug, Clone)]
pub(super) struct SearchStyle {
    pub(super) active: Style,
    pub(super) inactive: Style,
}

#[derive(Debug, Clone)]
pub(super) struct BlockStyle {
    pub(super) active: Style,
    pub(super) inactive: Style,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct StyleProperties {
    foreground: Option<Color>,
    background: Option<Color>,
}

#[derive(Debug, Clone)]
pub(super) struct HighlightStyle {
    pub(super) active: Style,
    pub(super) inactive: Style,
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
