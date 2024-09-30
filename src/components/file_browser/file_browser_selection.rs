use std::{
    fs,
    fs::DirEntry,
    path::PathBuf,
    cmp::Ordering,
};

use crate::{
    structs::Song,
    cue::CueSheet,
};

const VALID_EXTENSIONS: [&str; 8] = ["mp3", "mp4", "m4a", "wav", "flac", "ogg", "aac", "cue"];

#[derive(Debug, Clone)]
pub enum FileBrowserSelection {
    Song(Song),
    CueSheet(CueSheet),
    Directory(PathBuf),
}

impl FileBrowserSelection {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        if path.is_dir() {
            Some(FileBrowserSelection::Directory(path))
        } else if path.extension().is_some_and(|e| e == "cue") {
            CueSheet::from_file(&path).ok().map(FileBrowserSelection::CueSheet)
        } else {
            Song::from_file(&path).ok().map(FileBrowserSelection::Song)
        }
    }

    pub fn to_path(&self) -> PathBuf {
        match self {
            FileBrowserSelection::Song(s) => { s.path.clone() }
            FileBrowserSelection::CueSheet(cs) => { cs.cue_sheet_file_path() }
            FileBrowserSelection::Directory(p) => { p.clone() }
        }
    }
}

impl PartialEq for FileBrowserSelection {
    fn eq(&self, other: &Self) -> bool {
        match self {
            FileBrowserSelection::Directory(path) => {
                match other {
                    FileBrowserSelection::Directory(other_path) => path == other_path,
                    _ => false,
                }
            }
            FileBrowserSelection::CueSheet(cue_sheet) => {
                match other {
                    FileBrowserSelection::CueSheet(other_cue_sheet) =>
                        cue_sheet.cue_sheet_file_path() == other_cue_sheet.cue_sheet_file_path(),
                    _ => false,
                }
            }
            FileBrowserSelection::Song(song) => {
                match other {
                    FileBrowserSelection::Song(other_song) => song.path == other_song.path,
                    _ => false,
                }
            }
        }
    }
}

impl Eq for FileBrowserSelection {}

impl PartialOrd for FileBrowserSelection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self {
            FileBrowserSelection::Directory(path) => {
                // Directories come first
                match other {
                    FileBrowserSelection::Directory(other_path) => path.cmp(other_path),
                    _ => Ordering::Less,
                }
            }
            FileBrowserSelection::CueSheet(cue_sheet) => {
                // then queue sheets
                match other {
                    FileBrowserSelection::Directory(_) => Ordering::Greater,
                    FileBrowserSelection::CueSheet(other_cue_sheet) =>
                        cue_sheet.cue_sheet_file_path().cmp(&other_cue_sheet.cue_sheet_file_path()),
                    FileBrowserSelection::Song(_) => Ordering::Less,
                }
            }
            FileBrowserSelection::Song(song) => {
                // last, but not least, songs
                match other {
                    FileBrowserSelection::Directory(_) => Ordering::Greater,
                    FileBrowserSelection::CueSheet(_) => Ordering::Greater,
                    FileBrowserSelection::Song(other_song) => song.path.cmp(&other_song.path),
                }
            }
        })
    }
}

impl Ord for FileBrowserSelection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn dir_entry_to_file_browser_selection(entry: &DirEntry) -> Option<FileBrowserSelection>{
    if dir_entry_is_dir(&entry) {
        Some(FileBrowserSelection::Directory(entry.path()))
    } else if dir_entry_is_song(&entry) {
        Some(FileBrowserSelection::Song(Song::from_file(&entry.path()).unwrap()))
    } else if dir_entry_is_cue(&entry) {
        Some(FileBrowserSelection::CueSheet(CueSheet::from_file(&entry.path()).unwrap()))
    } else {
        None
    }
}

pub fn directory_to_songs_and_folders(path: &PathBuf) -> Vec<FileBrowserSelection> {
    let Ok(entries) = path.read_dir() else {
        return vec![];
    };

    let mut items: Vec<FileBrowserSelection> = entries
        .filter_map(|e| e.ok())
        .filter(|e| path_is_not_hidden(&e.path()))
        .filter_map(|e| dir_entry_to_file_browser_selection(&e))
        .collect();

    items.sort_unstable();
    items
}

pub fn dir_entry_is_file(dir_entry: &DirEntry) -> bool {
    // TODO: resolve symlinks
    dir_entry.file_type().is_ok_and(|ft| ft.is_file())
}

pub fn dir_entry_is_dir(dir_entry: &DirEntry) -> bool {
    let Ok(ft) = dir_entry.file_type() else {
        log::error!("dir_entry_is_dir: .file_type() returned error for {:?}", dir_entry.path());
        return false;
    };

    if ft.is_symlink() {
        let ln = fs::canonicalize(dir_entry.path());
        ln.is_ok_and(|ln| ln.is_dir())
    } else {
        ft.is_dir()
    }
}

pub fn path_is_not_hidden(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|e| e.to_str())
        .map(|e| e.to_string())
        .is_some_and(|d| !d.starts_with('.'))
}

pub fn dir_entry_has_song_extension(dir_entry: &DirEntry) -> bool {
    dir_entry
        .path()
        .extension()
        .is_some_and(|e| VALID_EXTENSIONS.contains(&e.to_str().unwrap()))
}

pub fn dir_entry_is_song(dir_entry: &DirEntry) -> bool {
    dir_entry_is_file(dir_entry) && dir_entry_has_song_extension(dir_entry)
}


pub fn dir_entry_has_cue_extension(dir_entry: &DirEntry) -> bool {
    dir_entry
        .path()
        .extension()
        .is_some_and(|e| e == "cue")
}

pub fn dir_entry_is_cue(dir_entry: &DirEntry) -> bool {
    dir_entry_is_file(dir_entry) && dir_entry_has_cue_extension(dir_entry)
}
