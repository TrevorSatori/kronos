use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;

use crate::toml::read_toml_file_or_default;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: Theme,
}

#[serde_inline_default::serde_inline_default]
#[derive(Serialize, Deserialize, Debug, Copy, Clone, DefaultFromSerde)]
pub struct Theme {
    #[serde_inline_default(Color::from_hsl(29.0, 34.0, 20.0))]
    pub top_bar_background: Color,

    #[serde_inline_default(Color::from_hsl(39.0, 67.0, 69.0))]
    pub top_bar_foreground_selected: Color,

    #[serde_inline_default(Color::from_hsl(29.0, 54.0, 61.0))]
    pub foreground: Color,

    #[serde_inline_default(Color::Black)]
    pub foreground_selected: Color,

    #[serde_inline_default(Color::White)]
    pub foreground_secondary: Color,

    #[serde_inline_default(Color::Black)]
    pub background: Color,

    #[serde_inline_default(Color::from_hsl(29.0, 54.0, 61.0))]
    pub background_selected: Color,

    #[serde_inline_default(Color::from_hsl(29.0, 54.0, 34.0))]
    pub background_selected_blur: Color,

    #[serde_inline_default(Color::Red)]
    pub search: Color,
}

impl Config {
    pub fn from_file() -> Self {
        read_toml_file_or_default("config")
    }
}
