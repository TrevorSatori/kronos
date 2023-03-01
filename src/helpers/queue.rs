use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use lofty::{AudioFile, Probe};
use tui::widgets::ListState;

use super::gen_funcs::bulk_add;
use crate::constants::{SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};

pub struct Queue {
    state: ListState,
    items: VecDeque<PathBuf>,
    curr: usize,
    total_time: u32,
}

impl Queue {
    pub fn with_items() -> Self {
        Self {
            state: ListState::default(),
            items: VecDeque::new(),
            curr: 0,
            total_time: 0,
        }
    }

    // return item at index
    pub fn item(&self) -> Option<&PathBuf> {
        if self.items.is_empty() {
            None
        } else {
            Some(&self.items[self.curr])
        }
    }

    // return all items contained in vector
    pub fn items(&self) -> &VecDeque<PathBuf> {
        &self.items
    }

    pub fn length(&self) -> usize {
        self.items.len()
    }

    pub fn total_time(&self) -> String {
        // days
        if self.total_time / SECONDS_PER_DAY >= 1 {
            let days = self.total_time / SECONDS_PER_DAY;
            let hours = (self.total_time % SECONDS_PER_DAY) / SECONDS_PER_HOUR;
            let minutes = (self.total_time % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;

            format!(
                " Total Length: {days} days {hours} hours {minutes} minutes |",
                days = days,
                hours = hours,
                minutes = minutes
            )
        // hours
        } else if self.total_time / SECONDS_PER_HOUR >= 1 {
            let hours = self.total_time / SECONDS_PER_HOUR;
            let minutes = (self.total_time % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
            let seconds = self.total_time % SECONDS_PER_MINUTE;

            format!(
                " Total Length: {hours} hours {minutes} minutes {seconds} seconds |",
                hours = hours,
                minutes = minutes,
                seconds = seconds
            )
        // minutes
        } else if self.total_time / SECONDS_PER_MINUTE >= 1 {
            let minutes = self.total_time / SECONDS_PER_MINUTE;
            let seconds = self.total_time % SECONDS_PER_MINUTE;

            format!(
                " Total Length: {minutes} minutes {seconds} seconds |",
                minutes = minutes,
                seconds = seconds
            )
        // seconds
        } else if self.total_time > 0 {
            format!(
                " Total Length: {total_time} seconds |",
                total_time = self.total_time
            )
        } else {
            "".to_string()
        }
    }

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
        let item = self.items[self.curr].clone();
        let length = self.item_length(&item);
        self.total_time -= length;
    }

    // get audio file length
    pub fn item_length(&mut self, path: &PathBuf) -> u32 {
        let path = Path::new(&path);
        let tagged_file = Probe::open(path)
            .expect("ERROR: Bad path provided!")
            .read()
            .expect("ERROR: Failed to read file!");

        let properties = &tagged_file.properties();
        let duration = properties.duration();

        // update song length, currently playing
        duration.as_secs() as u32
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
        self.curr = i;
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
        self.curr = i;
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
            self.items.remove(self.curr);
            self.unselect();
        // if at bottom of queue, remove item and select item above above
        } else if self.state.selected().unwrap() >= (self.items.len() - 1) {
            self.decrement_total_time();
            self.items.remove(self.curr);
            self.curr -= 1;
            self.state.select(Some(self.curr));
        // else delete item
        } else if !self.items.is_empty() {
            self.decrement_total_time();
            self.items.remove(self.curr);
        };
    }
}
