use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
struct ThemeToml {
    #[serde(default)]
    foreground: String,
    #[serde(default)]
    background: String,
    #[serde(default)]
    highlight_foreground: String,
    #[serde(default)]
    highlight_background: String,
}

impl Default for ThemeToml {
    fn default() -> Self {
        Self {
            foreground: "".to_string(),
            background: "".to_string(),
            highlight_foreground: "".to_string(),
            highlight_background: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigToml {
    #[serde(default)]
    theme: ThemeToml,
}

#[derive(Debug)]
struct Theme {
    foreground: Color,
    background: Color,
    highlight_foreground: Color,
    highlight_background: Color,
}

#[derive(Debug)]
pub struct Config {
    theme: Theme,
}

fn load_config_string() -> Option<(PathBuf, String)> {
    let config_paths = [home::home_dir().unwrap().as_path().join(".config/jolteon/config.toml")];

    for config in config_paths {
        let result: Result<String, std::io::Error> = fs::read_to_string(&config);

        if let Ok(file_content) = result {
            return Some((config, file_content));
        }
    }

    None
}

fn load_config_toml() -> ConfigToml {
    let config_string = load_config_string();

    let config_toml_option = match config_string {
        Some((path, content)) => match toml::from_str(&content) {
            Ok(toml) => Some(toml),
            Err(err) => {
                eprintln!(
                    "Could not parse {:?} as toml. Will use default values. Error was: \n{:?}",
                    path, err
                );
                None
            }
        },
        None => None,
    };

    config_toml_option.unwrap_or(ConfigToml {
        theme: ThemeToml::default(),
    })
}

impl Config {
    pub fn new() -> Self {
        let config_toml = load_config_toml();

        let color1 = Color::from_hsl(29.0, 54.0, 61.0);
        // let color2 = Color::from_hsl(39.0, 67.0, 69.0);

        let theme = Theme {
            foreground: Color::from_str(&config_toml.theme.foreground).unwrap_or(color1),
            background: Color::from_str(&config_toml.theme.background).unwrap_or(Color::Black),
            highlight_foreground: Color::from_str(&config_toml.theme.highlight_foreground).unwrap_or(Color::Black),
            highlight_background: Color::from_str(&config_toml.theme.highlight_background).unwrap_or(color1),
        };

        Self { theme }
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
