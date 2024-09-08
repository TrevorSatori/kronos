use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::{error};

use crate::{
    cue::CueSheet,
    structs::song::{directory_to_songs_and_folders, Song},
    ui::stateful_list::StatefulList,
};

pub struct Browser<'a> {
    pub items: StatefulList<String>,
    pub current_directory: PathBuf,
    pub filter: Option<String>,
    last_offset: usize,
    on_select_fn: Box<dyn FnMut((FileBrowserSelection, bool)) + 'a>,
}

#[allow(dead_code, unused_variables)]
#[derive(Debug)]
pub enum FileBrowserSelection {
    Song(Song),
    CueSheet(CueSheet),
    Directory,
}

impl<'a> Browser<'a> {
    pub fn new(current_directory: PathBuf) -> Self {
        let mut items = StatefulList::with_items(directory_to_songs_and_folders(&current_directory));
        items.select(0);

        Self {
            items,
            current_directory,
            filter: None,
            last_offset: 0,
            on_select_fn: Box::new(|_| {}) as _,
        }
    }

    pub fn selected_item(&self) -> PathBuf {
        if self.items.empty() {
            Path::new(&self.current_directory).into()
        } else {
            Path::join(&self.current_directory, Path::new(&self.items.item()))
        }
    }

    pub fn on_select(&mut self, cb: impl FnMut((FileBrowserSelection, bool)) + 'a) {
        self.on_select_fn = Box::new(cb);
    }

    fn enter_selection(&mut self, for_queue: bool) {
        let path = self.selected_item();

        if path.is_dir() {
            self.navigate_into();
        } else if path.extension().is_some_and(|e| e == "cue") {
            match CueSheet::from_file(&path) {
                Ok(cue_sheet) => {
                    (self.on_select_fn)((FileBrowserSelection::CueSheet(cue_sheet), for_queue));
                }
                Err(err) => {
                    error!("Filed to read CueSheet {:#?}", err);
                }
            }
        } else {
            match Song::from_file(&path) {
                Ok(song) => {
                    (self.on_select_fn)((FileBrowserSelection::Song(song), for_queue));
                },
                Err(err) => {
                    error!("Filed to read Song {:#?}", err);
                },
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
        let parent = self.current_directory.as_path().parent().unwrap().to_path_buf();
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

    pub fn on_key_event(&mut self, key: KeyEvent) {
        if !self.filter.is_some() {
            self.on_normal_key_event(key);
        } else {
            self.on_filter_key_event(key);
        }
    }

    fn on_normal_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => { self.enter_selection(false); },
            KeyCode::Backspace => self.navigate_up(),
            KeyCode::Down | KeyCode::Char('j') => self.items.next(),
            KeyCode::Up | KeyCode::Char('k') => self.items.previous(),
            KeyCode::PageUp => self.items.previous_by(5),
            KeyCode::PageDown => self.items.next_by(5),
            KeyCode::End => self.select_last(),
            KeyCode::Home => self.items.select(0),
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                self.filter = Some("".to_string());
            }
            KeyCode::Char('a') => {
                self.enter_selection(true);
                self.items.next();
            }
            _ => {}
        }
    }

    fn on_filter_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter if key.modifiers == KeyModifiers::ALT => {
                self.enter_selection(true);
                self.items.next();
            }
            KeyCode::Enter => {
                self.filter = None;
                self.enter_selection(false);
            }
            KeyCode::Esc => {
                self.filter = None;
            }
            KeyCode::Down => {
                self.select_next_match();
            }
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                self.select_next_match();
            }
            KeyCode::Up => {
                self.select_previous_match();
            }
            KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => {
                self.select_previous_match();
            }
            KeyCode::Backspace => {
                self.filter_delete();
            }
            KeyCode::Char(char) => {
                self.filter_append(char);
            }
            _ => {}
        }
    }
}
