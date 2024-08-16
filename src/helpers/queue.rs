use std::time::Duration;
use std::{collections::VecDeque, path::PathBuf};
use super::gen_funcs::{path_list_to_song_list, path_to_song, path_to_song_list, Song};

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
        let (songs, errors) = path_list_to_song_list(queue.iter().map(PathBuf::from).collect());

        if !errors.is_empty() {
            eprintln!("Failed to load some songs.\n{:?}", errors);
        }

        let total_time = song_list_to_duration(&songs);
        Self {
            items: songs,
            selected_item_index: None,
            total_time,
        }
    }

    pub fn songs(&self) -> &VecDeque<Song> {
        &self.items
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

    pub fn selected_song_index(&self) -> Option<usize> {
        if self.items.is_empty() {
            None
        } else {
            self.selected_item_index
        }
    }

    pub fn selected_song(&self) -> Option<Song> {
        self.selected_song_index().map(|i| self.items[i].clone())
    }

    fn refresh_total_time(&mut self) {
        self.total_time = song_list_to_duration(&self.items);
    }

    pub fn pop(&mut self) -> Song {
        let l = self.items.pop_front().unwrap();
        self.refresh_total_time();
        l
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        };
        self.selected_item_index = match self.selected_item_index {
            Some(i) => Some(std::cmp::min(i + 1, self.items.len() - 1)),
            None => Some(0),
        }
    }

    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        };
        self.selected_item_index = match self.selected_item_index {
            Some(i) => Some(if i > 0 { i - 1 } else { 0 }),
            None => Some(0),
        }
    }

    pub fn select_none(&mut self) {
        self.selected_item_index = None;
    }

    pub fn add(&mut self, path: PathBuf) {
        if path.is_dir() {
            let files = path_to_song_list(&path);
            self.items.append(&mut VecDeque::from(files));
        } else {
            match path_to_song(&path) {
                Ok(song) => self.items.push_back(song),
                Err(err) => {
                    eprintln!("Could not add {:?}. Error was {:?}", &path, err);
                }
            }
        }
        self.refresh_total_time();
    }

    pub fn remove_selected(&mut self) {
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
