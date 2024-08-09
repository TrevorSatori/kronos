use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use glob::glob;
use lofty::{Accessor, AudioFile, Probe, TaggedFileExt};

pub struct Song {
    pub path: PathBuf,
    pub length: std::time::Duration,
    pub artist: Option<String>,
}

pub fn path_to_song(path: PathBuf) -> Song {
    let path = Path::new(&path);
    let tagged_file = Probe::open(path)
        .expect("ERROR: Bad path provided!")
        .read()
        .expect("ERROR: Failed to read file!");

    let properties = &tagged_file.properties();
    let primary_tag = tagged_file.primary_tag().unwrap();
    let artist = primary_tag.artist();

    Song {
        path: PathBuf::from(path),
        length: properties.duration(),
        artist: artist.map(|s| s.to_string()),
    }
}

// converts queue items to what's displayed for user
pub fn audio_display(path: &PathBuf) -> String {
    let path = Path::new(&path);
    let tagged_file = Probe::open(path)
        .expect("ERROR: Bad path provided!")
        .read()
        .expect("ERROR: Failed to read file!");

    let ptag = tagged_file.primary_tag().unwrap();
    let artist = ptag.artist();

    // if filename
    if let Some(i) = tagged_file.primary_tag().unwrap().title() {
        // if artist data
        if let Some(j) = artist {
            format!("{artist} - {title}", artist = j, title = i)
        } else {
            i.into()
        }
    } else {
        path.file_name().unwrap().to_str().unwrap().to_string()
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

pub fn bulk_add(selected: &PathBuf) -> Vec<PathBuf> {
    let mut items = Vec::new();
    env::set_current_dir(selected).unwrap();

    for item in glob::glob("./*")
        .expect("Failed to read glob pattern")
        .flatten()
    {
        let current_dir = env::current_dir().unwrap();
        let join = Path::join(&current_dir, Path::new(&item));
        let ext = Path::new(&item).extension().and_then(OsStr::to_str);
        if ext.is_some()
            && (item.extension().unwrap() == "mp3"
                || item.extension().unwrap() == "mp4"
                || item.extension().unwrap() == "m4a"
                || item.extension().unwrap() == "wav"
                || item.extension().unwrap() == "flac"
                || item.extension().unwrap() == "ogg"
                || item.extension().unwrap() == "aac")
        {
            items.push(join);
        }
    }
    env::set_current_dir("../").unwrap();
    items
}
