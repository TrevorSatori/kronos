use serde::{Deserialize, Serialize};
use std::fs;
use toml;
use tui::style::Color;

#[derive(Serialize, Deserialize, Debug)]
struct Theme {
    foreground: Option<String>,
    background: Option<String>,
    highlight_foreground: Option<String>,
    highlight_background: Option<String>,
}

// for tables
#[derive(Serialize, Deserialize, Debug)]
struct ConfigTOML {
    theme: Option<Theme>,
}

// everything
#[derive(Debug)]
pub struct Config {
    foreground: Color,
    background: Color,
    highlight_foreground: Color,
    highlight_background: Color,
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
        let config_toml: ConfigTOML = toml::from_str(&content).unwrap_or_else(|_| {
            // if config file not found, set defaults
            eprintln!("FAILED TO CREATE CONFIG OBJECT FROM FILE");
            ConfigTOML { theme: None }
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
        }
    }

    // pub fn get_quit(&self) -> KeyCode {

    //     KeyCode::Char(self.quit)
    // }

    // pub fn get_play_pause(&self) -> char {
    //     self.play_pause
    // }

    // pub fn get_skip(&self) -> char {
    //     self.skip
    // }

    // pub fn get_queue_add(&self) -> char {
    //     self.queue_add
    // }

    // pub fn get_queue_remove(&self) -> char {
    //     self.queue_remove
    // }

    pub fn get_foreground(&self) -> Color {
        self.foreground
    }

    pub fn get_background(&self) -> Color {
        self.background
    }

    pub fn get_highlight_foreground(&self) -> Color {
        self.highlight_foreground
    }

    pub fn get_highlight_background(&self) -> Color {
        self.highlight_background
    }
}
