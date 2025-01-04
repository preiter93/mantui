#![allow(unused)]
use ratatheme::{DeserializeTheme, Subtheme};

use std::error::Error;
use std::sync::OnceLock;
use tachyonfx::HslConvertable;

use ratatui::style::{Color, Style};

#[must_use]
pub(super) fn get_theme() -> &'static Theme {
    THEME.get_or_init(Theme::default)
}

static THEME: OnceLock<Theme> = OnceLock::new();

#[derive(Debug, Clone, DeserializeTheme)]
pub(super) struct Theme {
    #[theme(style)]
    pub(super) base: Style,

    #[theme(styles(active, selected, inactive))]
    pub(super) list: ListStyle,

    #[theme(styles(active, inactive))]
    pub(super) search: ActivatableStyle,

    #[theme(styles(active, inactive))]
    pub(super) block: ActivatableStyle,

    #[theme(styles(active, inactive))]
    pub(super) highlight: ActivatableStyle,
}

impl Default for Theme {
    fn default() -> Self {
        let toml_str = r##"
        [colors]
        "gray300" = "#c0c0d8"
        "gray500" = "#454554"
        "gray700" = "#1c1c21"
        "orange" = "#ff9900"
        "charcoal" = "#1c1c20"
        "red500" = "hsl(337, 100%, 48%)"
        "red700" = "hsl(337, 70%, 38%)"

        [base]
        foreground = "white"

        [list]
        active.foreground = "white"
        inactive.foreground = "gray500"
        selected.foreground = "charcoal"
        selected.background = "red500"

        [search]
        active.foreground = "white"
        inactive.foreground = "gray500"

        [highlight]
        active.foreground = "black"
        active.background = "red500"
        inactive.foreground = "black"
        inactive.background = "red700"

        [block]
        active.foreground = "white"
        inactive.foreground = "gray500"
    "##;

        let deserializer = toml::Deserializer::new(toml_str);
        Self::deserialize_theme(deserializer).unwrap()
    }
}

#[derive(Debug, Default, Clone, Subtheme)]
pub(super) struct ListStyle {
    #[theme(style)]
    pub(super) active: Style,

    #[theme(style)]
    pub(super) inactive: Style,

    #[theme(style)]
    pub(super) selected: Style,
}

#[derive(Debug, Default, Clone, Subtheme)]
pub(super) struct ActivatableStyle {
    #[theme(style)]
    pub(super) active: Style,

    #[theme(style)]
    pub(super) inactive: Style,
}
