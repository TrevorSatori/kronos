use std::fs;

use serde::{Deserialize, Serialize};
use tui::style::Color;

#[derive(Serialize, Deserialize, Debug)]
struct Theme {
    foreground: Option<String>,
    background: Option<String>,
    highlight_foreground: Option<String>,
    highlight_background: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Layout {
    progress_bar: Option<u16>,
}

// for tables
#[derive(Serialize, Deserialize, Debug)]
struct ConfigToml {
    theme: Option<Theme>,
    layout: Option<Layout>,
}

// everything
#[derive(Debug)]
pub struct Config {
    foreground: Color,
    background: Color,
    highlight_foreground: Color,
    highlight_background: Color,
    progress_bar: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        // may want to add more path options later
        let config_paths = [home::home_dir()
            .unwrap()
            .as_path()
            .join(".config/kronos/config.toml")];

        // placeholder to store config in
        let mut content: String = "".to_owned();

        // for filepaths in above array, check to see if there is a config
        for config in config_paths {
            let result: Result<String, std::io::Error> = fs::read_to_string(config);

            if let Ok(file_content) = result {
                content = file_content;
                break;
            }
        }

        // convert toml file to serialized data
        let config_toml: ConfigToml = toml::from_str(&content).unwrap_or_else(|_| {
            // if config file not found, set defaults
            eprintln!("FAILED TO CREATE CONFIG OBJECT FROM FILE");
            ConfigToml {
                theme: None,
                layout: None,
            }
        });

        // match theme
        let (foreground, background, highlight_foreground, highlight_background) = match config_toml
            .theme
        {
            // 200, 100, 255
            Some(theme) => {
                // item, if error
                let map = |i: Option<String>, s: String| {
                    let rgb = i.clone();
                    match i.unwrap_or(s).to_ascii_lowercase().as_ref() {
                        "black" => Color::Black,
                        "blue" => Color::Blue,
                        "green" => Color::Green,
                        "red" => Color::Red,
                        "yellow" => Color::Yellow,
                        "magenta" => Color::Magenta,
                        "cyan" => Color::Cyan,
                        "gray" => Color::Gray,
                        "dark gray" => Color::DarkGray,
                        "light red" => Color::LightRed,
                        "light green" => Color::LightGreen,
                        "light yellow" => Color::LightYellow,
                        "light blue" => Color::LightBlue,
                        "light magenta" => Color::LightMagenta,
                        "light cyan" => Color::LightCyan,
                        "white" => Color::White,
                        _ => {
                            let colors: Vec<u8> = rgb.unwrap()
                            .split(|i| i == ',')
                            .map(|i| i.to_string().trim().parse().expect("Couldn't read RGB Values. Make sure each value is between 0 & 255"))
                            .collect();

                            if colors.len() == 3 {
                                Color::Rgb(colors[0], colors[1], colors[2])
                            } else {
                                eprintln!("Couldn't read RGB Values. Make sure each value is comma seperated");
                                Color::Black
                            }
                        }
                    }
                };

                let foreground = map(theme.foreground, "LightCyan".to_string());
                let background = map(theme.background, "Black".to_string());
                let hfg = map(theme.highlight_foreground, "Black".to_string());
                let hbg = map(theme.highlight_background, "Light Cyan".to_string());

                (foreground, background, hfg, hbg)
            }

            None => (
                Color::LightCyan,
                Color::Black,
                Color::Black,
                Color::LightCyan,
            ),
        };

        let progress_bar = match config_toml.layout {
            Some(i) => i.progress_bar.unwrap_or(35),
            None => 35,
        };

        Self {
            // quit: quit, // gathered from above
            // play_pause: play_pause,
            // skip: skip,
            // queue_add: queue_add,
            // queue_remove: queue_remove,
            foreground,
            background,
            highlight_foreground,
            highlight_background,
            progress_bar,
        }
    }

    // pub fn quit_key(&self) -> KeyCode {

    //     KeyCode::Char(self.quit)
    // }

    // pub fn play_pause_key(&self) -> char {
    //     self.play_pause
    // }

    // pub fn skip_key(&self) -> char {
    //     self.skip
    // }

    // pub fn queue_add_key(&self) -> char {
    //     self.queue_add
    // }

    // pub fn queue_remove_key(&self) -> char {
    //     self.queue_remove
    // }

    pub fn foreground(&self) -> Color {
        self.foreground
    }

    pub fn background(&self) -> Color {
        self.background
    }

    pub fn highlight_foreground(&self) -> Color {
        self.highlight_foreground
    }

    pub fn highlight_background(&self) -> Color {
        self.highlight_background
    }

    pub fn progress_bar(&self) -> u16 {
        self.progress_bar
    }
}
