use lofty::{TaggedFileExt};
use ratatui::widgets::ListState;
use std::time::Duration;
use std::{
    collections::VecDeque,
    path::{PathBuf},
};

use super::gen_funcs::{bulk_add, path_to_song, Song};

pub struct Queue {
    state: ListState,
    items: VecDeque<Song>,
    selected_item_index: usize,
    total_time: Duration,
}

fn song_list_to_duration(items: &VecDeque<Song>) -> Duration {
    items.iter().map(|s| s.length).sum()
}

impl Queue {
    pub fn new(queue: Vec<String>) -> Self {
        let items: VecDeque<Song> = queue.iter().map(PathBuf::from).map(path_to_song).collect();
        let total_time = song_list_to_duration(&items);
        Self {
            state: ListState::default(),
            items,
            selected_item_index: 0,
            total_time,
        }
    }

    pub fn state(&self) -> ListState {
        self.state.clone()
    }

    pub fn length(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn paths(&self) -> VecDeque<PathBuf> {
        self.items.iter().map(|i| i.path.clone()).collect()
    }

    pub fn total_time(&self) -> Duration {
        self.total_time
    }

    pub fn selected_item_path(&self) -> Option<&PathBuf> {
        if self.items.is_empty() {
            None
        } else {
            Some(&self.items[self.selected_item_index].path)
        }
    }

    fn refresh_total_time(&mut self) {
        self.total_time = song_list_to_duration(&self.items);
    }

    pub fn pop(&mut self) -> PathBuf {
        let l = self.items.pop_front().unwrap().path;
        self.refresh_total_time();
        l
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
                self.items.push_back(song);
            }
        } else {
            let song = path_to_song(item);
            self.items.push_back(song);
        }
        self.refresh_total_time();
    }

    pub fn remove(&mut self) {
        if self.items.is_empty() {
            return;
        }

        self.items.remove(self.selected_item_index);

        if self.items.is_empty() {
            self.unselect();
        } else {
            self.selected_item_index = self.selected_item_index.min(self.items.len() - 1);
            self.state.select(Some(self.selected_item_index));
        }

        self.refresh_total_time();
    }
}
