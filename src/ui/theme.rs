use ratatui::style::{Color, Style};
use serde::Deserialize;
use std::sync::OnceLock;
use tui_theme_builder::ThemeBuilder;

#[derive(Debug, Deserialize)]
pub struct Colors {
    pub white: Color,
    pub black: Color,
    pub gray: Color,
    pub charcoal: Color,
    pub red: Color,
    pub darkred: Color,
}

impl Default for Colors {
    fn default() -> Self {
        let s = r##"
        "white" = "white"
        "black" = "#101116"
        "gray" = "#454554"
        "orange" = "#ff9900"
        "charcoal" = "#1c1c20"
        "red" = "#f5005e"
        "darkred" = "#a51d51"
        "##;
        toml::from_str(s).unwrap()
    }
}

#[must_use]
pub(super) fn get_theme() -> &'static Theme {
    THEME.get_or_init(Theme::default)
}

static THEME: OnceLock<Theme> = OnceLock::new();

#[derive(Debug, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct Theme {
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

#[derive(Debug, Default, Clone, ThemeBuilder)]
#[builder(context=Colors)]
pub(super) struct ListStyle {
    #[style(fg=white)]
    pub(super) active: Style,

    #[style(fg=gray)]
    pub(super) inactive: Style,

    #[style(fg=charcoal, bg=red)]
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
    #[style(fg=black, bg=red)]
    pub(super) active: Style,

    #[style(fg=black, bg=darkred)]
    pub(super) inactive: Style,
}
