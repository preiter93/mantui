#![allow(unused)]
use serde::Deserialize;
use tui_theme_builder::ThemeBuilder;

use std::error::Error;
use std::sync::OnceLock;
use tachyonfx::HslConvertable;

use ratatui::style::{Color, Style};

#[derive(Debug, Deserialize)]
pub struct Colors {
    pub white: Color,
    pub black: Color,
    pub gray300: Color,
    pub gray500: Color,
    pub gray700: Color,
    pub orange: Color,
    pub charcoal: Color,
    pub red500: Color,
    pub red700: Color,
}

impl Default for Colors {
    fn default() -> Self {
        let s = r##"
        "white" = "white"
        "black" = "black"
        "gray300" = "#c0c0d8"
        "gray500" = "#454554"
        "gray700" = "#1c1c21"
        "orange" = "#ff9900"
        "charcoal" = "#1c1c20"
        "red500" = "#f5005e"
        "red700" = "#a51d51"
        "##;
        toml::from_str(s).unwrap()
    }
}

#[must_use]
pub(super) fn get_theme() -> &'static Theme {
    THEME.get_or_init(Theme::default)
}

static THEME: OnceLock<Theme> = OnceLock::new();

#[derive(Debug, Clone)]
pub(super) struct Theme {
    pub(super) base: BaseStyle,

    pub(super) list: ListStyle,

    pub(super) search: SearchStyle,

    pub(super) block: BlockStyle,

    pub(super) highlight: HighlightStyle,
}

impl Default for Theme {
    fn default() -> Self {
        let colors = Colors::default();
        let base = BaseStyle::build(&colors);
        let list = ListStyle::build(&colors);
        let search = SearchStyle::build(&colors);
        let block = BlockStyle::build(&colors);
        let highlight = HighlightStyle::build(&colors);
        Self {
            base,
            list,
            search,
            block,
            highlight,
        }
    }
}

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct ListStyle {
    #[style(fg=white)]
    pub(super) active: Style,

    #[style(fg=gray500)]
    pub(super) inactive: Style,

    #[style(fg=charcoal, bg=red500)]
    pub(super) selected: Style,
}

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct BaseStyle {
    #[style(fg=white)]
    pub(super) style: Style,
}

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct SearchStyle {
    #[style(fg=white)]
    pub(super) active: Style,

    #[style(fg=gray500)]
    pub(super) inactive: Style,
}

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct BlockStyle {
    #[style(fg=white)]
    pub(super) active: Style,

    #[style(fg=gray500)]
    pub(super) inactive: Style,
}

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct HighlightStyle {
    #[style(fg=black, bg=red500)]
    pub(super) active: Style,

    #[style(fg=black, bg=red700)]
    pub(super) inactive: Style,
}
