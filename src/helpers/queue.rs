use lofty::{TaggedFileExt};
use ratatui::widgets::ListState;
use std::time::Duration;
use std::{
    collections::VecDeque,
    path::{PathBuf},
};

use super::gen_funcs::{bulk_add, path_to_song, Song};

pub struct Queue {
    items: VecDeque<Song>,
    selected_item_index: Option<usize>,
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
            items,
            selected_item_index: None,
            total_time,
        }
    }

    pub fn state(&self) -> ListState {
        ListState::default().with_selected(self.selected_item_index)
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

    pub fn selected_song(&self) -> Option<Song> {
        if self.items.is_empty() {
            None
        } else {
            self.selected_item_index.map(|i| self.items[i].clone())
        }
    }

    pub fn selected_item_path(&self) -> Option<PathBuf> {
        self.selected_song().map(|i| i.path)
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
        self.selected_item_index = match self.selected_item_index {
            Some(i) => Some(std::cmp::min(i + 1, self.items.len() - 1)),
            None => Some(0),
        }
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        };
        self.selected_item_index = match self.selected_item_index {
            Some(i) => Some(if i > 0 { i - 1 } else { 0 }),
            None => Some(0),
        }
    }

    pub fn unselect(&mut self) {
        self.selected_item_index = None;
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
        if let Some(selected_item_index) = self.selected_item_index {
            self.items.remove(selected_item_index);

            if self.items.is_empty() {
                self.selected_item_index = None;
            } else {
                self.selected_item_index = Some(selected_item_index.min(self.items.len() - 1));
            }

            self.refresh_total_time();
        }


    }
}
