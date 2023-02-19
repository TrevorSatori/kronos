use std::{
    path::{Path, PathBuf},
    thread::{self},
    time::Duration,
};
extern crate glob;
use crate::lib::stateful_list::*;
use crate::lib::{gen_funcs, music_handler::MusicHandle, queue::Queue, stateful_table::*};
use std::env;

#[derive(Clone, Copy)]
pub enum InputMode {
    Browser,
    Queue,
    Controls,
}

pub struct App<'a> {
    pub browser_items: StatefulList<String>,
    pub queue_items: Queue,
    pub control_table: StatefulTable<'a>,
    pub music_handle: MusicHandle,
    input_mode: InputMode,
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            browser_items: StatefulList::with_items(gen_funcs::scan_folder()),
            queue_items: Queue::with_items(),
            control_table: StatefulTable::new(),
            music_handle: MusicHandle::new(),
            input_mode: InputMode::Browser,
            titles: vec!["Music", "Controls"],
            index: 0,
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn get_input_mode(&self) -> InputMode {
        self.input_mode.clone()
    }

    pub fn set_input_mode(&mut self, in_mode: InputMode) {
        self.input_mode = in_mode
    }

    pub fn get_current_song(&self) -> String {
        if self.music_handle.sink_empty() && self.queue_items.is_empty() {
            "CURRENT SONG".to_string()
        } else {
            self.music_handle.get_currently_playing()
        }
    }

    // if item selected is folder, enter folder, else play record.
    pub fn evaluate(&mut self) {
        let join = self.selected_item();
        // if folder enter, else play song
        if join.is_dir() {
            env::set_current_dir(join).unwrap();
            self.browser_items = StatefulList::with_items(gen_funcs::scan_folder());
            self.browser_items.next();
        } else {
            self.music_handle.play(join);
        }
    }

    // cd into selected directory
    pub fn backpedal(&mut self) {
        env::set_current_dir("../").unwrap();
        self.browser_items = StatefulList::with_items(gen_funcs::scan_folder());
        self.browser_items.next();
    }

    // if queue has items and nothing playing, auto play
    pub fn auto_play(&mut self) {
        thread::sleep(Duration::from_millis(250));
        if self.music_handle.sink_empty() && !self.queue_items.is_empty() {
            self.music_handle.set_time_played(0);
            self.music_handle.play(self.queue_items.pop());
        }
    }

    // if playing and
    pub fn song_progress(&mut self) -> u16 {
        let progress = || {
            let percentage =
                (self.music_handle.get_time_played() * 100) / self.music_handle.get_song_length();
            if percentage >= 100 {
                100
            } else {
                percentage
            }
        };

        // edge case if nothing queued or playing
        if self.music_handle.sink_empty() && self.queue_items.is_empty() {
            0

        // if something playing, calculate progress
        } else if !self.music_handle.sink_empty() {
            progress()
        // if nothing playing keep rolling
        } else {
            self.auto_play();
            0
        }
    }

    // get file path
    pub fn selected_item(&self) -> PathBuf {
        let current_dir = env::current_dir().unwrap();
        if self.browser_items.empty() {
            return Path::new(&current_dir).into();
        } else {
            let join = Path::join(&current_dir, Path::new(&self.browser_items.get_item()));
            join
        }
    }
}
