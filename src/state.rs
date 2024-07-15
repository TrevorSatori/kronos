use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StateToml {
    pub last_visited_path: Option<String>,
}

pub fn load_state() -> StateToml {
    let state_file_paths = [home::home_dir()
        .unwrap()
        .as_path()
        .join(".config/kronos/state.toml")];

    let mut content: String = "".to_owned();

    for state_file_path in state_file_paths {
        let result: Result<String, std::io::Error> = fs::read_to_string(state_file_path);

        if let Ok(file_content) = result {
            content = file_content;
            break;
        }
    }

    let state_toml: StateToml = toml::from_str(&content).unwrap_or_else(|_| {
        eprintln!("FAILED TO CREATE STATE OBJECT FROM FILE");
        StateToml {
            last_visited_path: None,
        }
    });

    state_toml
}
pub fn save_state(state: &StateToml) {
    let state_file_path = home::home_dir()
        .unwrap()
        .as_path()
        .join(".config/kronos/state.toml");

    if let Ok(serialized) = toml::to_string(state) {
        fs::write(state_file_path, serialized);
        // TODO: not ignore IO result
    }
}