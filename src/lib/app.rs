use std::{path::{PathBuf, Path}, thread::{self}, time::{Duration}}; 
extern crate glob;
use std::env;
use crate::lib::stateful_list::*;
use super::{queue::Queue, gen_funcs};
use super::music_handler::{MusicHandle};

// app is responsible for handling state
// keeps track of which Field you are in (QUEUE, Browser)
// updates and handles list state

#[derive(Clone, Copy)]
pub enum InputMode {
    Browser,
    Queue,
}

pub struct App {
    pub browser_items: StatefulList<String>,
    pub queue_items: Queue,
    pub music_handle: MusicHandle,
    input_mode: InputMode,
}

impl App {

    pub fn new() -> App {
        App {
            browser_items: StatefulList::with_items(gen_funcs::scan_folder()),
            queue_items: Queue::with_items(),
            music_handle: MusicHandle::new(),
            input_mode: InputMode::Browser,
        }
    }

    pub fn get_input_mode(&self) -> InputMode{
        self.input_mode.clone()
    }

    pub fn set_input_mode(&mut self, in_mode: InputMode){
        self.input_mode = in_mode
    }

    // if item selected is folder, enter folder, else play record.
    pub fn evaluate(&mut self){
        let join = self.selected_item();
        // if folder enter, else play song
        if join.is_dir() {
            env::set_current_dir(join).unwrap();
            self.browser_items = StatefulList::with_items(gen_funcs::scan_folder());
        } else {
            self.music_handle.play(join);
        }
    }

    // cd into selected directory
    pub fn backpedal(&mut self){
      env::set_current_dir("../").unwrap();
      self.browser_items = StatefulList::with_items(gen_funcs::scan_folder());
      self.browser_items.next();
    }


    // if queue has items and nothing playing, auto play
    pub fn auto_play(&mut self){
        thread::sleep(Duration::from_millis(250));
        if self.music_handle.get_sink_length() == 0 && !self.queue_items.is_empty() {
            self.play_next();
        }
    }

    // should pop item from queue and play next
    pub fn play_next(&mut self){
        self.music_handle.set_time_played(0);
        match self.queue_items.pop() {
            Some(item) => self.music_handle.play(item.0),
            None => (),
        }
    }

    // if playing and 
    pub fn song_progress(&mut self) -> u16 { 
        let progress = || {
            let percentage = (self.music_handle.get_time_played() * 100) / self.music_handle.get_song_length();
            if percentage >= 100 {
                100
            } else {
                percentage
            }
        };

        // edge case if nothing queued or playing
        if self.music_handle.get_sink_length() == 0 && self.queue_items.is_empty() {
            0

        // if something playing, calculate progress 
        } else if self.music_handle.get_sink_length() == 1 {
            progress()
        // if nothing playing keep rolling
        } else {
          self.auto_play();
          0
        }
                    
    }
    // get file path
    pub fn selected_item(&self) -> PathBuf{
        let current_dir = env::current_dir().unwrap();
        let join = Path::join(&current_dir, Path::new(&self.browser_items.get_item()));
        join
    }  

}

