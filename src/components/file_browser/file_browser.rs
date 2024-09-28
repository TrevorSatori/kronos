use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crossterm::event::{KeyCode, KeyEvent};
use log::error;

use crate::{
    cue::CueSheet,
    structs::{Song, Queue},
    config::{Theme},
};

const VALID_EXTENSIONS: [&str; 8] = ["mp3", "mp4", "m4a", "wav", "flac", "ogg", "aac", "cue"];

pub struct FileBrowser<'a> {
    pub(super) theme: Theme,
    pub(super) items: Vec<String>,
    current_directory: PathBuf,
    pub(super) filter: Option<String>,
    last_offset: usize,
    on_select_fn: Box<dyn FnMut((FileBrowserSelection, KeyEvent)) + 'a>,
    pub(super) queue_items: Arc<Queue>,
    pub(super) selected_index: usize,
    pub(super) height: Mutex<usize>,
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
        Self {
            theme,
            items: directory_to_songs_and_folders(&current_directory),
            current_directory,
            filter: None,
            last_offset: 0,
            on_select_fn: Box::new(|_| {}) as _,
            queue_items,
            selected_index: 0,
            height: Mutex::new(0),
        }
    }

    pub fn filter(&self) -> &Option<String> {
        &self.filter
    }

    pub fn blur(&mut self) {

    }

    pub fn focus(&mut self) {

    }

    pub fn current_directory(&self) -> &PathBuf {
        &self.current_directory
    }

    pub fn selected_item(&self) -> PathBuf {
        if self.items.is_empty() {
            self.current_directory.clone()
        } else {
            self.current_directory.join(&self.items[self.selected_index])
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
                    self.select_next();

                }
                Err(err) => {
                    error!("Filed to read CueSheet {:#?}", err);
                }
            }
        } else {
            match Song::from_file(&path) {
                Ok(song) => {
                    (self.on_select_fn)((FileBrowserSelection::Song(song), key_event));
                    self.select_next();
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
            // self.last_offset = self.offset;
            self.items = directory_to_songs_and_folders(&path);
            self.select_next();
        }
    }

    pub fn navigate_up(&mut self) {
        let Some(parent) = self.current_directory.as_path().parent().map(|p| p.to_path_buf()) else { return };
        self.items = directory_to_songs_and_folders(&parent);
        self.select_by_path();
        // self.offset = self.last_offset;
        self.current_directory = parent;
    }

    pub fn select_next(&mut self) {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.selected_index = self.selected_index.saturating_add(1);
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn select_last(&mut self) {
        self.selected_index = self.items.len().saturating_sub(1).max(0);
    }

    pub fn next_index_wrapped(&self, i: usize) -> usize {
        if i >= self.items.len() - 1 {
            0
        } else {
            i + 1
        }
    }

    pub fn previous_index_wrapped(&self, i: usize) -> usize {
        if i == 0 {
            self.items.len() - 1
        } else {
            i - 1
        }
    }

    pub fn find_by_path(&self, s: &PathBuf) -> usize {
        let mut i = 0;

        for n in 0..self.items.len() {
            if s.ends_with(self.items[n].to_string()) {
                i = n;
                break;
            }
        }

        i
    }

    pub fn select_by_path(&mut self) {
        self.selected_index = self.find_by_path(&self.current_directory);
    }

    pub fn find_next_by_match(&self, s: &str, direction_forward: bool) -> Option<usize> {
        let mut i: usize = self.selected_index;

        loop {
            i = if direction_forward {
                self.next_index_wrapped(i)
            } else {
                self.previous_index_wrapped(i)
            };

            if i == self.selected_index {
                return None;
            }

            if self.items[i].to_string().to_lowercase().contains(&s.to_lowercase()) {
                return Some(i);
            }
        }
    }

    pub fn select_next_match(&mut self) {
        if let Some(ref s) = self.filter {
            if let Some(i) = self.find_next_by_match(s, true) {
                self.selected_index = i;
            }
        }
    }

    pub fn select_previous_match(&mut self) {
        if let Some(ref s) = self.filter {
            if let Some(i) = self.find_next_by_match(s, false) {
                self.selected_index = i;
            }
        }
    }

    pub fn select_next_by_match(&mut self, s: &str) {
        if let Some(i) = self.find_next_by_match(s, true) {
            self.selected_index = i;
        }
    }

    pub fn filter_append(&mut self, char: char) {
        if let Some(filter) = self.filter.as_mut() {
            filter.push(char);
        } else {
            self.filter = Some(char.to_string());
        }

        if !self
            .selected_item()
            .to_string_lossy()
            .to_lowercase()
            .contains(&self.filter.clone().unwrap().to_lowercase())
        {
            self.select_next_by_match(&self.filter.clone().unwrap());
        }
    }

    pub fn filter_delete(&mut self) {
        self.filter = match &self.filter {
            Some(s) if s.len() > 0 => Some(s[..s.len() - 1].to_string()), // TODO: s[..s.len()-1] can panic! use .substring crate
            _ => None,
        };
    }

}
