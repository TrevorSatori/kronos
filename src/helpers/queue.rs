use lofty::{TaggedFileExt};
use ratatui::widgets::ListState;
use std::time::Duration;
use std::{
    collections::VecDeque,
    path::{PathBuf},
};

use super::gen_funcs::{bulk_add, path_to_song, Song};

pub fn read_song_length(path: PathBuf) -> Duration {
    path_to_song(path).length
}

pub struct Queue {
    state: ListState,
    items: VecDeque<Song>,
    selected_item_index: usize,
    total_time: Duration,
}

impl Queue {
    pub fn new(queue: Vec<String>) -> Self {
        let items: VecDeque<Song> = queue.iter().map(PathBuf::from).map(path_to_song).collect();
        let total_time: Duration = items.iter().map(|s| s.length).sum();
        Self {
            state: ListState::default(),
            items,
            selected_item_index: 0,
            total_time,
        }
    }

    pub fn item(&self) -> Option<&PathBuf> {
        if self.items.is_empty() {
            None
        } else {
            Some(&self.items[self.selected_item_index].path)
        }
    }

    pub fn paths(&self) -> VecDeque<PathBuf> {
        self.items.iter().map(|i| i.path.clone()).collect()
    }

    pub fn length(&self) -> usize {
        self.items.len()
    }

    pub fn total_time(&self) -> Duration {
        self.total_time
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn pop(&mut self) -> PathBuf {
        self.decrement_total_time();
        self.items.pop_front().unwrap().path
    }

    pub fn state(&self) -> ListState {
        self.state.clone()
    }

    fn decrement_total_time(&mut self) {
        let length = self.items[self.selected_item_index].length;
        self.total_time = self.total_time.saturating_sub(length);
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        };

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_item_index = i;
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        };
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_item_index = i;
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn add(&mut self, item: PathBuf) {
        if item.is_dir() {
            let files = bulk_add(&item);
            for f in files {
                let song = path_to_song(f);
                self.total_time += song.length;
                self.items.push_back(song);
            }
        } else {
            let song = path_to_song(item);
            self.total_time += song.length;
            self.items.push_back(song);
        }
    }

    pub fn remove(&mut self) {
        if self.items.is_empty() {
            // top of queue
        } else if self.items.len() == 1 {
            self.decrement_total_time();
            self.items.remove(self.selected_item_index);
            self.unselect();
        // if at bottom of queue, remove item and select item above above
        } else if self.state.selected().unwrap() >= (self.items.len() - 1) {
            self.decrement_total_time();
            self.items.remove(self.selected_item_index);
            self.selected_item_index -= 1;
            self.state.select(Some(self.selected_item_index));
        // else delete item
        } else if !self.items.is_empty() {
            self.decrement_total_time();
            self.items.remove(self.selected_item_index);
        };
    }
}
