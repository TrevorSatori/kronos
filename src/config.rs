use std::{fs};
use serde::{Serialize, Deserialize};
use toml;


#[derive(Serialize, Deserialize, Debug)]
struct ConfigControls{
    quit: Option<char>,
    play_pause: Option<char>,
    skip: Option<char>,
    queue_add: Option<char>,
    queue_remove: Option<char>,
}

pub struct Config{
    quit: char,
    play_pause: char,
    skip: char,
    queue_add: char,
    queue_remove: char,
}

// for tables
#[derive(Serialize, Deserialize, Debug)]
struct ConfigTOML{

    controls: Option<ConfigControls>, 
    // TODO add colorscheme
}

// cd
// parent dir
// select list item (up Down)
// switch grid focus (Left, right)
// chage tabs 



impl Config{
    pub fn new() -> Config {

        let config_paths = [
            "./config.toml",
            "./Config.toml", 
            "~/.config/kronos/config.toml",
            "~/.config/kronos/Config.toml",
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

        // print content
        // println!("{:?}", content);

        // 
        let config_toml: ConfigTOML = toml::from_str(&content).unwrap_or_else(|_|{
            // if config file not found, set defaults
            println!("FAILED TO CREATE CONFIG OBJECT FROM FILE");
            ConfigTOML{
                controls: None
            }
        });

        // convert found controls to variables
        let (quit, play_pause, skip, queue_add, queue_remove) = match config_toml.controls {
            Some(controls) => {
                let quit = controls.quit.unwrap_or_else(|| {
                    'u'
                });
                (quit)
            },
            None => {
                println!("Missing data"); 
                'q'
            },  
        };

        Config {  
            quit: quit, // gathered from above 
            play_pause: 'p',
            skip: 'g',
            queue_add: 'a',
            queue_remove: 'r',
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


}
