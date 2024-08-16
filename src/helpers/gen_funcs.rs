use glob::glob;
use lofty::{Accessor, AudioFile, Probe, TaggedFileExt};
use std::fs::DirEntry;
use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct Song {
    pub path: PathBuf,
    pub length: std::time::Duration,
    pub artist: Option<String>,
    pub title: Option<String>,
}

pub fn path_to_song(path: PathBuf) -> Song {
    let path = Path::new(&path);
    let tagged_file = Probe::open(path)
        .expect("ERROR: Bad path provided!")
        .read()
        .expect("ERROR: Failed to read file!");

    let properties = &tagged_file.properties();

    let (artist, title) = match tagged_file.primary_tag() {
        Some(primary_tag) => {
            (primary_tag.artist().map(String::from), primary_tag.title().map(String::from))
        },
        _ => (None, None),
    };

    Song {
        path: PathBuf::from(path),
        length: properties.duration(),
        artist,
        title,
    }
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

pub fn scan_and_filter_directory() -> Vec<String> {
    let mut items = Vec::new();
    let valid_extensions: HashSet<&str> = ["mp3", "mp4", "m4a", "wav", "flac", "ogg", "aac"]
        .iter()
        .cloned()
        .collect();

    let current_dir = env::current_dir().expect("Failed to get current directory");

    // Use glob instead of glob_with, which uses default match options
    for entry in glob("./*").expect("Failed to read glob pattern").flatten() {
        let path = current_dir.join(&entry);

        if path.is_dir() {
            if let Some(file_name) = path.file_name() {
                if !file_name.to_str().unwrap_or_default().starts_with('.') {
                    items.push(entry.to_str().unwrap().to_owned());
                }
            }
        } else if let Some(ext) = path.extension().and_then(OsStr::to_str) {
            if valid_extensions.contains(ext) {
                items.push(entry.to_str().unwrap().to_owned());
            }
        }
    }
    items
}

const extensions: [&str; 7] = ["mp3", "mp4", "m4a", "wav", "flac", "ogg", "aac"];

fn dir_entry_is_file(dir_entry: &DirEntry) -> bool {
    dir_entry.file_type().is_ok_and(|ft| ft.is_file())
}

fn dir_entry_has_song_extension(dir_entry: &DirEntry) -> bool {
    dir_entry
        .path()
        .extension()
        .is_some_and(|e| extensions.contains(&e.to_str().unwrap()))
}

fn dir_entry_is_song(dir_entry: &DirEntry) -> bool {
    dir_entry_is_file(dir_entry) && dir_entry_has_song_extension(dir_entry)
}

pub fn path_to_song_list(path: &PathBuf) -> Vec<Song> {
    let mut entries = match path.read_dir() {
        Ok(files) => files
            .filter_map(|file| file.ok())
            .filter(dir_entry_is_song)
            .map(|dir_entry| dir_entry.path())
            .map(path_to_song)
            .collect(),
        _ => vec![],
    };
    entries.sort_unstable_by_key(|i| i.path.clone());
    entries
}
