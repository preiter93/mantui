use ratatui::style::{Color, Style};
use serde::Deserialize;
use std::sync::OnceLock;
use tui_theme_builder::ThemeBuilder;

use crate::args::Args;

#[derive(Debug, Deserialize)]
pub struct Colors {
    pub white: Color,
    pub black: Color,
    pub gray: Color,
    pub orange: Color,
}

impl Default for Colors {
    fn default() -> Self {
        let s = r##"
        "white" = "white"
        "black" = "#101116"
        "gray" = "#454554"
        "lightorange" = "#e0af67"
        "orange" = "#f89a63"
        "charcoal" = "#1c1c20"
        "##;
        toml::from_str(s).unwrap()
    }
}

#[must_use]
pub(super) fn get_theme() -> &'static Theme {
    THEME.get().unwrap()
}

pub static THEME: OnceLock<Theme> = OnceLock::new();

#[derive(Debug, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub struct Theme {
    #[style(fg=white,bg=black)]
    pub(super) base: Style,

    pub(super) list: ListStyle,

    pub(super) search: SearchStyle,

    pub(super) block: BlockStyle,

    pub(super) highlight: HighlightStyle,
}

impl Default for Theme {
    fn default() -> Self {
        let colors = Colors::default();
        Self::build(&colors)
    }
}

impl Theme {
    pub fn init(args: &Args) -> Self {
        let mut theme = Self::default();
        if args.transparent {
            theme.base = Style::default().fg(theme.base.fg.unwrap());
        }
        theme
    }
}

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct ListStyle {
    #[style(fg=white)]
    pub(super) active: Style,

    #[style(fg=gray)]
    pub(super) inactive: Style,

    #[style(fg=black, bg=orange)]
    pub(super) selected: Style,
}

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct SearchStyle {
    #[style(fg=white)]
    pub(super) active: Style,

    #[style(fg=gray)]
    pub(super) inactive: Style,
}

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct BlockStyle {
    #[style(fg=white)]
    pub(super) active: Style,

    #[style(fg=gray)]
    pub(super) inactive: Style,
}

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct HighlightStyle {
    #[style(fg=black, bg=orange)]
    pub(super) active: Style,

    #[style(fg=black, bg=orange)]
    pub(super) inactive: Style,
}
