use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    structs::Queue,
    config::{Theme},
};

use super::file_browser_selection::{
    FileBrowserSelection,
    directory_to_songs_and_folders,
};

pub struct FileBrowser<'a> {
    on_select_fn: Box<dyn FnMut((FileBrowserSelection, KeyEvent)) + 'a>,

    current_directory: PathBuf,
    pub(super) items: Vec<FileBrowserSelection>,
    pub(super) selected_index: usize,
    pub(super) filter: Option<String>,

    pub(super) queue_items: Arc<Queue>,

    pub(super) theme: Theme,
    padding: usize,
    pub(super) height: Mutex<usize>,
    pub(super) offset: usize,
}

impl<'a> FileBrowser<'a> {
    pub fn new(theme: Theme, current_directory: PathBuf, queue_items: Arc<Queue>) -> Self {
        let items = directory_to_songs_and_folders(&current_directory);

        Self {
            on_select_fn: Box::new(|_| {}) as _,

            current_directory,
            items,
            selected_index: 0,
            filter: None,

            queue_items,

            theme,
            padding: 6,
            height: Mutex::new(0),
            offset: 0,
        }
    }

    pub fn filter(&self) -> &Option<String> {
        &self.filter
    }

    pub fn blur(&mut self) {
        log::warn!("FileBrowser.blur() unimplemented");
    }

    pub fn focus(&mut self) {
        log::warn!("FileBrowser.focus() unimplemented");
    }

    pub fn current_directory(&self) -> &PathBuf {
        &self.current_directory
    }

    pub fn selected_item(&self) -> FileBrowserSelection {
        if self.items.is_empty() {
            log::error!("self.selected_index -> self.items.is_empty()");
            FileBrowserSelection::from_path(&self.current_directory).unwrap()
        } else if self.selected_index >= self.items.len() {
            log::error!("self.selected_index >= self.items.len()");
            FileBrowserSelection::from_path(&self.current_directory).unwrap()
        } else {
            self.items[self.selected_index].clone()
            // self.current_directory.join(&self.items[self.selected_index])
        }
    }

    pub fn on_select(&mut self, cb: impl FnMut((FileBrowserSelection, KeyEvent)) + 'a) {
        self.on_select_fn = Box::new(cb);
    }

    pub(super) fn enter_selection(&mut self, key_event: KeyEvent) {
        let fbs = self.selected_item();

        match fbs {
            FileBrowserSelection::Directory(path) if key_event.code == KeyCode::Enter => {
                self.navigate_into(path);
            }
            _ => {
                (self.on_select_fn)((fbs, key_event));
            }
        }
    }

    pub fn navigate_into(&mut self, path: PathBuf) {
        self.current_directory = path.clone();
        self.items = directory_to_songs_and_folders(&path);
        self.selected_index = 0;
    }

    pub fn navigate_up(&mut self) {
        let Some(parent) = self.current_directory.as_path().parent().map(|p| p.to_path_buf()) else { return };
        self.items = directory_to_songs_and_folders(&parent);
        self.select_current_directory();
        self.current_directory = parent;
    }

    pub fn select_next(&mut self) {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.selected_index = self.selected_index.saturating_add(1);

            if self.selected_index > self.offset + self.padding_bottom() {
                self.set_offset(self.selected_index, self.padding_bottom());
            }
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);

            if self.selected_index < self.offset + self.padding_top() {
                self.set_offset(self.selected_index, self.padding_top());
            }
        }
    }

    pub fn select_first(&mut self) {
        self.selected_index = 0;
        self.set_offset(self.selected_index, self.padding_bottom());
    }

    pub fn select_last(&mut self) {
        self.selected_index = self.items.len().saturating_sub(1).max(0);
        self.set_offset(self.selected_index, self.padding_top());
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
            if s.ends_with(self.items[n].to_path().as_path()) {
                i = n;
                break;
            }
        }

        i
    }

    pub fn select_current_directory(&mut self) {
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

            if self.items[i].to_path().to_string_lossy().to_lowercase().contains(&s.to_lowercase()) {
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
            .to_path()
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



    fn padding_top(&self) -> usize {
        6
    }

    fn padding_bottom(&self) -> usize {
        self.height.lock().unwrap().saturating_sub(self.padding)
    }

    pub fn set_offset(&mut self, i: usize, padding: usize) {
        self.offset = if i > padding {
            i.saturating_sub(padding).min(self.items.len().saturating_sub(*self.height.lock().unwrap()))
        } else {
            0
        };
    }

}

impl Drop for FileBrowser<'_> {
    fn drop(&mut self) {
        log::trace!("FileBrowser.drop()");
    }
}
