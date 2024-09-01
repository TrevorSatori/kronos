use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::toml::{read_toml_file_or_default};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: Theme,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct Theme {
    #[serde(default = "default_foreground")]
    pub foreground: Color,

    #[serde(default = "default_background")]
    pub background: Color,

    #[serde(default = "default_highlight_foreground")]
    pub highlight_foreground: Color,

    #[serde(default = "default_highlight_background")]
    pub highlight_background: Color,

    #[serde(default = "default_search")]
    pub search: Color,

    #[serde(default = "default_top_bar_background")]
    pub top_bar_background: Color,

    #[serde(default = "default_top_bar_highlight")]
    pub top_bar_highlight: Color,
}

fn default_foreground() -> Color { Color::from_hsl(29.0, 54.0, 61.0) }
fn default_background() -> Color { Color::Black }
fn default_highlight_foreground() -> Color { Color::Black }
fn default_highlight_background() -> Color { Color::from_hsl(29.0, 54.0, 61.0) }
fn default_search() -> Color { Color::Red }
fn default_top_bar_background() -> Color { Color::from_hsl(29.0, 34.0, 20.0) }
fn default_top_bar_highlight() -> Color { Color::from_hsl(39.0, 67.0, 69.0) }

impl Config {
    pub fn from_file() -> Self {
        read_toml_file_or_default("config")
    }
}
