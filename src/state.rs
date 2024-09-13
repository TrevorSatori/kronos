use serde::{Deserialize, Serialize};

use crate::{
    structs::Song,
    ui::Playlist,
    toml::{read_toml_file_or_default, write_toml_file, TomlFileError},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub last_visited_path: Option<String>,
    #[serde(default)]
    pub queue_items: Vec<Song>,
    #[serde(default)]
    pub playlists: Vec<Playlist>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            last_visited_path: None,
            queue_items: vec![],
            playlists: vec![],
        }
    }
}

impl State {
    pub fn from_file() -> Self {
        read_toml_file_or_default("state")
    }

    pub fn to_file(&self) -> Result<(), TomlFileError> {
        write_toml_file("state", self)
    }
}
