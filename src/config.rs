use std::{fs, clone, ascii::AsciiExt};
use serde::{Serialize, Deserialize};
use toml;
use tui::style::Color;

// Controls
#[derive(Serialize, Deserialize, Debug)]
struct ConfigControls{
    quit: Option<char>,
    play_pause: Option<char>,
    skip: Option<char>,
    queue_add: Option<char>,
    queue_remove: Option<char>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Theme{
    foreground: Option<String>,
    background: Option<String>, 
    highlight_foreground: Option<String>,
    highlight_background: Option<String>,
}


// for tables
#[derive(Serialize, Deserialize, Debug)]
struct ConfigTOML{
    controls: Option<ConfigControls>,
    theme: Option<Theme>, 
}

// cd
// parent dir
// select list item (up Down)
// switch grid focus (Left, right)
// chage tabs 
// # Black, Red, Yellow, Blue, Magenta, Green, Cyan, Gray, DarkGray, LightRed, LightGreen
// LightYellow, LightBlue, LightMagenta, LightCyan
 
// everything
#[derive(Debug)]
pub struct Config{
    quit: char,
    play_pause: char,
    skip: char,
    queue_add: char,
    queue_remove: char,
    foreground: Color,
    background: Color,
    highlight_foreground: Color, 
    highlight_background: Color,
}


impl Config{
    pub fn new() -> Config {

        let config_paths = [
            "./config.toml".to_ascii_lowercase(),
            "~/.config/kronos/config.toml".to_ascii_lowercase(),
        ];

        // placeholder to store config in
        let mut content: String = "".to_owned();

        // for filepaths in above array, check to see if there is a config
        for config in config_paths{
            let result: Result<String, std::io::Error> = fs::read_to_string(config);

            if result.is_ok() {
                content = result.unwrap();
                break;
            }
        }

        // convert toml file to serialized data
        let config_toml: ConfigTOML = toml::from_str(&content).unwrap_or_else(|_|{
            // if config file not found, set defaults
            println!("FAILED TO CREATE CONFIG OBJECT FROM FILE");
            ConfigTOML{
                controls: None,
                theme: None,
            }
        });

        // convert found controls to variables
        let (quit, play_pause, skip, queue_add, queue_remove) = match config_toml.controls {
            Some(controls) => {

                let quit = controls.quit.unwrap_or_else(|| {
                    'q'
                });

                let play_pause = controls.play_pause.unwrap_or_else(|| {
                    'p'
                });

                let skip = controls.skip.unwrap_or_else(|| {
                    'g'
                });

                let queue_add = controls.queue_add.unwrap_or_else(|| {
                    'a'
                });

                let queue_remove = controls.queue_remove.unwrap_or_else(|| {
                    'r'
                });

                (quit, play_pause, skip, queue_add, queue_remove)
            },
            // if 0 fields filled out
            None => {
                println!("Missing data"); 
                ('q', 'p', 'g', 'a', 'r' )
            },  
        };

        // match theme
        let (foreground, background, hfg, hbg) = match config_toml.theme {

            Some(theme) => {
                // item, if error, _value
                let map = |i: Option<String>, s: String| {
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
                        _ => Color::Black,
                    }
                };

                let foreground = map(theme.foreground, "LightCyan".to_string());
                let background = map(theme.background, "Black".to_string());
                let hfg = map(theme.highlight_foreground, "Black".to_string());
                let hbg = map(theme.highlight_background, "Light Cyan".to_string());

                (foreground, background, hfg, hbg)
            }, 

            None => {
                (Color::LightCyan, Color::Black, Color::Black, Color::LightCyan)
            }, 
            
        }; 
 
        Config {  
            quit: quit, // gathered from above 
            play_pause: play_pause,
            skip: skip,
            queue_add: queue_add,
            queue_remove: queue_remove,
            foreground: foreground,
            background: background,
            highlight_foreground: hfg, 
            highlight_background: hbg,
        }
    }

    pub fn get_quit(&self) -> char {
        self.quit
    }

    pub fn get_play_pause(&self) -> char {
        self.play_pause
    }

    pub fn get_skip(&self) -> char {
        self.skip
    }

    pub fn get_queue_add(&self) -> char {
        self.queue_add
    }

    pub fn get_queue_remove(&self) -> char {
        self.queue_remove
    }

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
