use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};
use std::time::Duration;
use lofty::{AudioFile, Probe};
use ratatui::widgets::ListState;

use super::gen_funcs::bulk_add;

pub struct Queue {
    state: ListState,
    items: VecDeque<PathBuf>,
    selected_item_index: usize,
    total_time: Duration,
}

impl Queue {
    pub fn new(queue: Vec<String>) -> Self {
        let items = queue.iter().map(PathBuf::from).collect();
        Self {
            state: ListState::default(),
            items,
            selected_item_index: 0,
            total_time: Duration::from_secs(0),
        }
    }

    // return item at index
    pub fn item(&self) -> Option<&PathBuf> {
        if self.items.is_empty() {
            None
        } else {
            Some(&self.items[self.selected_item_index])
        }
    }

    // return all items contained in vector
    pub fn items(&self) -> &VecDeque<PathBuf> {
        &self.items
    }

    pub fn length(&self) -> usize {
        self.items.len()
    }

    pub fn total_time(&self) -> Duration { self.total_time }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn pop(&mut self) -> PathBuf {
        self.decrement_total_time();
        self.items.pop_front().unwrap()
    }

    pub fn state(&self) -> ListState {
        self.state.clone()
    }

    fn decrement_total_time(&mut self) {
        // TODO:
        //   1. store song length for playing file + all queue files in RAM
        //   2. do "refresh queue length", deterministic, rather than "decrement_total_time"
        // eprintln!("decrement_total_time {:?} / {:?}", self.selected_item_index, self.items.len());
        let item = self.items[self.selected_item_index].clone();
        let length = self.item_length(&item);
        self.total_time = self.total_time.saturating_sub(length);
    }

    pub fn item_length(&self, path: &PathBuf) -> Duration {
        let path = Path::new(&path);
        let tagged_file = Probe::open(path)
            .expect("ERROR: Bad path provided!")
            .read()
            .expect("ERROR: Failed to read file!");

        let properties = &tagged_file.properties();
        properties.duration()
    }

    pub fn next(&mut self) {
        // check if empty
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
        // check if empty
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
                let length = self.item_length(&f);
                self.total_time += length;
                self.items.push_back(f);
            }
        } else {
            self.total_time += self.item_length(&item);
            self.items.push_back(item);
        }
    }

    // remove item from items vector
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
