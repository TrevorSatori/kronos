use std::{
    collections::VecDeque,
    fs::DirEntry,
    path::{Path, PathBuf},
};

use lofty::{Accessor, AudioFile, LoftyError, Probe, TaggedFileExt};

#[derive(Clone)]
pub struct Song {
    pub path: PathBuf,
    pub length: std::time::Duration,
    pub artist: Option<String>,
    pub title: Option<String>,
}

const VALID_EXTENSIONS: [&str; 7] = ["mp3", "mp4", "m4a", "wav", "flac", "ogg", "aac"];

pub fn path_to_song(path: &PathBuf) -> Result<Song, LoftyError> {
    let path = Path::new(path);
    let tagged_file = Probe::open(path)?.read()?;

    let (artist, title) = match tagged_file.primary_tag() {
        Some(primary_tag) => (
            primary_tag.artist().map(String::from),
            primary_tag.title().map(String::from),
        ),
        _ => (None, None),
    };

    Ok(Song {
        path: PathBuf::from(path),
        length: tagged_file.properties().duration(),
        artist,
        title,
    })
}

pub fn song_to_string(song: &Song) -> String {
    if let Some(title) = &song.title {
        if let Some(artist) = &song.artist {
            format!("{artist} - {title}")
        } else {
            title.into()
        }
    } else {
        song.path.file_name().unwrap().to_str().unwrap().to_string()
    }
}

pub fn scan_and_filter_directory(path: &PathBuf) -> Vec<String> {
    let entries = path.read_dir().unwrap();

    let mut items: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|entry| dir_entry_is_dir(&entry) || (dir_entry_is_file(&entry) && dir_entry_has_song_extension(&entry)))
        .map(|entry| entry.path())
        .filter(path_is_not_hidden)
        .filter_map(|path| path.file_name().and_then(|e| e.to_str()).map(|e| e.to_string()))
        .collect();

    items.sort_unstable();
    items
}

fn dir_entry_is_file(dir_entry: &DirEntry) -> bool {
    dir_entry.file_type().is_ok_and(|ft| ft.is_file())
}

fn dir_entry_is_dir(dir_entry: &DirEntry) -> bool {
    dir_entry.file_type().is_ok_and(|ft| ft.is_dir())
}

fn path_is_not_hidden(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|e| e.to_str())
        .map(|e| e.to_string())
        .is_some_and(|d| !d.starts_with('.'))
}

fn dir_entry_has_song_extension(dir_entry: &DirEntry) -> bool {
    dir_entry
        .path()
        .extension()
        .is_some_and(|e| VALID_EXTENSIONS.contains(&e.to_str().unwrap()))
}

fn dir_entry_is_song(dir_entry: &DirEntry) -> bool {
    dir_entry_is_file(dir_entry) && dir_entry_has_song_extension(dir_entry)
}

pub fn path_to_song_list(path: &PathBuf) -> Vec<Song> {
    match path.read_dir() {
        Ok(read_dir) => {
            let paths = read_dir
                .filter_map(|file| file.ok())
                .filter(dir_entry_is_song)
                .map(|dir_entry| dir_entry.path())
                .collect();
            let (songs, errors) = path_list_to_song_list(paths);

            if !errors.is_empty() {
                eprintln!("Could not add some songs: {:?}", errors);
            }

            let mut songs = Vec::from(songs);
            songs.sort_unstable_by_key(|i| i.path.clone());
            songs
        }
        _ => Vec::new(),
    }
}

pub fn path_list_to_song_list(paths: Vec<PathBuf>) -> (VecDeque<Song>, VecDeque<(PathBuf, LoftyError)>) {
    let mut songs: VecDeque<Song> = VecDeque::new();
    let mut errors: VecDeque<(PathBuf, LoftyError)> = VecDeque::new();

    for path in paths {
        match path_to_song(&path) {
            Ok(song) => {
                songs.push_back(song);
            }
            Err(err) => {
                errors.push_back((path, err));
            }
        };
    }

    (songs, errors)
}
