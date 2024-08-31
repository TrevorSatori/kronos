use std::fmt::{Formatter, Pointer};
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use log::error;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Config {
    #[serde(default)]
    pub theme: Theme,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Theme {
    #[serde(default)]
    pub foreground: Color,
    #[serde(default)]
    pub background: Color,
    #[serde(default)]
    pub highlight_foreground: Color,
    #[serde(default)]
    pub highlight_background: Color,
}

impl Default for Theme {
    // let color1 = Color::from_hsl(29.0, 54.0, 61.0);
    // let color2 = Color::from_hsl(39.0, 67.0, 69.0);
    fn default() -> Self {
        Self {
            foreground: Color::from_hsl(29.0, 54.0, 61.0),
            background: Color::Black,
            highlight_foreground: Color::Black,
            highlight_background: Color::from_hsl(29.0, 54.0, 61.0),
        }
    }
}

fn load_config_string() -> Option<(PathBuf, String)> {
    let config_paths = [home::home_dir()?.as_path().join(".config/jolteon/config.toml")];

    for config in config_paths {
        let result: Result<String, std::io::Error> = fs::read_to_string(&config);

        if let Ok(file_content) = result {
            return Some((config, file_content));
        }
    }

    None
}

fn load_config_toml() -> Config {
    let config_string = load_config_string();

    let config_toml_option = match config_string {
        Some((path, content)) => match toml::from_str(&content) {
            Ok(toml) => Some(toml),
            Err(err) => {
                error!(
                    "Could not parse {:?} as toml. Will use default values. Error was: \n{:?}",
                    path, err
                );
                None
            }
        },
        None => None,
    };

    config_toml_option.unwrap_or(Config {
        theme: Theme::default(),
    })
}

impl Config {
    pub fn new() -> Self {
        let config_toml = load_config_toml();

        Self { theme: config_toml.theme }
    }

    pub fn foreground(&self) -> Color {
        self.theme.foreground
    }

    pub fn background(&self) -> Color {
        self.theme.background
    }

    pub fn highlight_foreground(&self) -> Color {
        self.theme.highlight_foreground
    }

    pub fn highlight_background(&self) -> Color {
        self.theme.highlight_background
    }
}
