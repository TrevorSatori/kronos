use serde::{Deserialize, Serialize};

use crate::structs::Song;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
}

impl Playlist {
    fn _new(name: String) -> Self {
        Self {
            name,
            songs: vec![],
        }
    }
}
