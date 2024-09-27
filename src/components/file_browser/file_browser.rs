use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use log::error;

use crate::{
    cue::CueSheet,
    structs::{Song, Queue},
    ui::stateful_list::StatefulList,
    config::{Theme},
};

const VALID_EXTENSIONS: [&str; 8] = ["mp3", "mp4", "m4a", "wav", "flac", "ogg", "aac", "cue"];

pub struct FileBrowser<'a> {
    pub(super) theme: Theme,
    pub(super) items: StatefulList<String>,
    current_directory: PathBuf,
    pub(super) filter: Option<String>,
    last_offset: usize,
    on_select_fn: Box<dyn FnMut((FileBrowserSelection, KeyEvent)) + 'a>,
    pub(super) queue_items: Arc<Queue>,
}

#[allow(dead_code, unused_variables)]
#[derive(Debug)]
pub enum FileBrowserSelection {
    Song(Song),
    CueSheet(CueSheet),
    Directory(PathBuf),
}

fn directory_to_songs_and_folders(path: &PathBuf) -> Vec<String> {
    // TODO: .cue
    let Ok(entries) = path.read_dir() else {
        return vec![];
    };

    let mut items: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|entry| dir_entry_is_dir(&entry) || dir_entry_is_song(&entry))
        .map(|entry| entry.path())
        .filter(path_is_not_hidden)
        .filter_map(|path| path.file_name().and_then(|e| e.to_str()).map(|e| e.to_string()))
        .collect();

    items.sort_unstable();
    items
}

fn dir_entry_is_file(dir_entry: &DirEntry) -> bool {
    // TODO: resolve symlinks
    dir_entry.file_type().is_ok_and(|ft| ft.is_file())
}

fn dir_entry_is_dir(dir_entry: &DirEntry) -> bool {
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

impl<'a> FileBrowser<'a> {
    pub fn new(theme: Theme, current_directory: PathBuf, queue_items: Arc<Queue>) -> Self {
        let mut items = StatefulList::with_items(directory_to_songs_and_folders(&current_directory));
        items.select(0);

        Self {
            theme,
            items,
            current_directory,
            filter: None,
            last_offset: 0,
            on_select_fn: Box::new(|_| {}) as _,
            queue_items,
        }
    }

    pub fn items(&self) -> &StatefulList<String> {
        &self.items
    }

    pub fn filter(&self) -> &Option<String> {
        &self.filter
    }

    pub fn blur(&mut self) {
        self.items.unselect();
    }

    pub fn focus(&mut self) {
        self.items.next();
    }

    pub fn current_directory(&self) -> &PathBuf {
        &self.current_directory
    }

    pub fn selected_item(&self) -> PathBuf {
        if self.items.empty() {
            Path::new(&self.current_directory).into()
        } else {
            Path::join(&self.current_directory, Path::new(&self.items.item()))
        }
    }

    pub fn on_select(&mut self, cb: impl FnMut((FileBrowserSelection, KeyEvent)) + 'a) {
        self.on_select_fn = Box::new(cb);
    }

    pub(super) fn enter_selection(&mut self, key_event: KeyEvent) {
        let path = self.selected_item();

        if path.is_dir() {
            if key_event.code == KeyCode::Enter {
                self.navigate_into();
            } else {
                (self.on_select_fn)((FileBrowserSelection::Directory(path), key_event));
            }
        } else if path.extension().is_some_and(|e| e == "cue") {
            match CueSheet::from_file(&path) {
                Ok(cue_sheet) => {
                    (self.on_select_fn)((FileBrowserSelection::CueSheet(cue_sheet), key_event));
                    self.items.next();
                }
                Err(err) => {
                    error!("Filed to read CueSheet {:#?}", err);
                }
            }
        } else {
            match Song::from_file(&path) {
                Ok(song) => {
                    (self.on_select_fn)((FileBrowserSelection::Song(song), key_event));
                    self.items.next();
                }
                Err(err) => {
                    error!("Failed to read Song {:#?}", err);
                }
            }
        }
    }

    pub fn navigate_into(&mut self) {
        let path = self.selected_item();

        if path.is_dir() {
            self.current_directory = path.clone();
            self.last_offset = self.items.offset;
            self.items = StatefulList::with_items(directory_to_songs_and_folders(&path));
            self.items.next();
        }
    }

    pub fn navigate_up(&mut self) {
        let Some(parent) = self.current_directory.as_path().parent().map(|p| p.to_path_buf()) else { return };
        self.items = StatefulList::with_items(directory_to_songs_and_folders(&parent));
        self.items.select_by_path(&self.current_directory);
        self.items.offset = self.last_offset;
        self.current_directory = parent;
    }

    pub fn select_last(&mut self) {
        self.items.select(self.items.items().len() - 1)
    }

    pub fn select_next_match(&mut self) {
        if let Some(s) = &self.filter {
            self.items.select_next_by_match(s)
        }
    }

    pub fn select_previous_match(&mut self) {
        if let Some(s) = &self.filter {
            self.items.select_previous_by_match(s)
        }
    }

    pub fn filter_delete(&mut self) {
        self.filter = match &self.filter {
            Some(s) if s.len() > 0 => Some(s[..s.len() - 1].to_string()), // TODO: s[..s.len()-1] can panic! use .substring crate
            _ => None,
        };
    }

    pub fn filter_append(&mut self, char: char) {
        self.filter = match &self.filter {
            Some(s) => Some(s.to_owned() + char.to_string().as_str()),
            _ => Some(char.to_string()),
        };
        if !self
            .items
            .item()
            .to_lowercase()
            .contains(&self.filter.clone().unwrap().to_lowercase())
        {
            self.items.select_next_by_match(&self.filter.clone().unwrap());
        }
    }

}
