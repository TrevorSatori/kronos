use std::{
    path::PathBuf,
    time::Duration,
};

use lofty::{Accessor, AudioFile, LoftyError, Probe, TaggedFileExt};
use serde::{Deserialize, Serialize};

use crate::{
    cue::CueSheet,
    components::{FileBrowserSelection, directory_to_songs_and_folders},
};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Song {
    pub path: PathBuf,
    pub start_time: Duration,
    pub length: Duration,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track: Option<u32>,
}

impl Song {
    pub fn from_file(path: &PathBuf) -> Result<Self, LoftyError> {
        let tagged_file = Probe::open(path)?.read()?;

        let (artist, album, title, track) = match tagged_file.primary_tag() {
            Some(primary_tag) => (
                primary_tag.artist().map(String::from),
                primary_tag.album().map(String::from),
                primary_tag.title().map(String::from),
                primary_tag.track(),
            ),
            _ => (None, None, None, None),
        };

        Ok(Song {
            path: PathBuf::from(path),
            start_time: Duration::ZERO,
            length: tagged_file.properties().duration(),
            title: title.unwrap_or(path.file_name().unwrap().to_str().unwrap().to_string()),
            artist,
            album,
            track,
        })
    }

    pub fn from_dir(path: &PathBuf) -> Vec<Self> {
        let songs = directory_to_songs_and_folders(path);
        songs.iter().filter_map(|s| {
            if let FileBrowserSelection::Song(song) = s {
                Some(song.clone())
            } else {
                None
            }
        }).collect()
    }

    pub fn from_cue_sheet(cue_sheet: CueSheet) -> Vec<Self> {
        // log::debug!(target: "::song.from_cue_sheet", "{:?}", cue_sheet);

        let cue_file = cue_sheet.file().unwrap();
        let file_name = cue_file.name();
        let tracks = cue_file.tracks();

        let cue_path = cue_sheet.cue_sheet_file_path();
        let song_path = cue_path.parent().unwrap().join(file_name);

        let s = Song::from_file(&song_path).expect("could not load file");

        let mut songs: Vec<Song> = tracks
            .iter()
            .map(|t| Song {
                path: song_path.clone(),
                length: Duration::ZERO,
                artist: t.performer(),
                title: t.title(),
                start_time: t.start_time(),
                album: cue_sheet.title(),
                track: t.index().split_whitespace().nth(0).map(|i| i.parse().ok()).flatten(),
            })
            .collect();

        for i in 0..songs.len() {
            let next_start = if i < songs.len() - 1 {
                songs[i + 1].start_time
            } else {
                s.length
            };
            let this_start = songs[i].start_time;
            songs[i].length = next_start.saturating_sub(this_start);
        }

        songs
    }
}
