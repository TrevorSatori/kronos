use serde::{Deserialize, Serialize};

use crate::{
    structs::{Song, Playlist},
    toml::{read_toml_file_or_default, write_toml_file, TomlFileError},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub songs: Vec<Song>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            songs: vec![],
        }
    }
}

impl Library {
    pub fn from_file() -> Self {
        read_toml_file_or_default("library")
    }

    pub fn to_file(&self) -> Result<(), TomlFileError> {
        write_toml_file("library", self)
    }
}
