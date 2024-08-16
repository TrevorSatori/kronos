use glob::glob;
use lofty::{Accessor, AudioFile, LoftyError, Probe, TaggedFileExt};
use std::fs::DirEntry;
use std::{
    collections::{HashSet, VecDeque},
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

pub fn path_to_song(path: &PathBuf) -> Result<Song, LoftyError> {
    let path = Path::new(path);
    let tagged_file = Probe::open(path)?.read()?;

    let (artist, title) = match tagged_file.primary_tag() {
        Some(primary_tag) => {
            (primary_tag.artist().map(String::from), primary_tag.title().map(String::from))
        },
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
        },
        _ => Vec::new(),
    }

}

pub fn path_list_to_song_list(paths: Vec<PathBuf>) -> (VecDeque<Song>, VecDeque<(PathBuf, LoftyError)>) {
    let mut songs: VecDeque<Song> = VecDeque::new();
    let mut errors: VecDeque<(PathBuf, LoftyError)> = VecDeque::new();

    for path in paths {
        match path_to_song(&path) {
            Ok(song) => { songs.push_back(song); }
            Err(err) => { errors.push_back(( path, err )); }
        };
    }

    (songs, errors)
}