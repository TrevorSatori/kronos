use std::fs::{create_dir_all, read_to_string, write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub last_visited_path: Option<String>,
    #[serde(default)]
    pub queue_items: Vec<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            last_visited_path: None,
            queue_items: vec![],
        }
    }
}

pub fn load_state() -> Result<State, Box<dyn std::error::Error>> {
    let state_file_path = get_state_file_path()?;
    let content = read_to_string(state_file_path)?;
    let state = toml::from_str(&content)?;
    Ok(state) // why do we need to `?` and `Ok(state)`?
}

fn get_config_dir_path() -> Result<std::path::PathBuf, String> {
    match home::home_dir() {
        Some(path) => Ok(path.as_path().join(".config/jolteon")),
        _ => Err(String::from("Could not get home dir!")),
    }
}

fn get_state_file_path() -> Result<std::path::PathBuf, String> {
    get_config_dir_path().map(|p| p.join("state.toml"))
}

fn create_dir() -> Result<(), Box<dyn std::error::Error>> {
    create_dir_all(get_config_dir_path()?)?;
    Ok(())
}

pub fn save_state(state: &State) -> Result<(), Box<dyn std::error::Error>> {
    create_dir()?;
    let state_file_path = get_state_file_path()?;
    let serialized = toml::to_string(state)?;
    write(state_file_path, serialized)?;
    Ok(())
}
