use std::{fs, path::{PathBuf, Path}, thread::{self, JoinHandle}, sync::{Arc, Mutex}, time::{Duration, Instant, self}, rc::Rc}; 
extern crate glob;
use glob::{glob, glob_with, MatchOptions, Pattern};
use std::env;

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};
use std::fs::File;
use std::io::BufReader;
use rodio::{Sink, Decoder, OutputStream, source::Source, OutputStreamHandle, queue::SourcesQueueOutput};
use std::ffi::OsStr;
use metadata::MediaFileMetadata;


use crate::lib::stateful_list::*;
use super::{queue::Queue, music_handler};
use super::music_handler::{MusicHandle};

// app is responsible for handling state //
// keeps track of which Field you are in (QUEUE, Browser)
// updates and handles list state

pub enum InputMode {
    Browser,
    Queue,
}


pub struct App {
    pub browser_items: StatefulList<String>,
    pub queue_items: Queue,
    pub input_mode: InputMode,
    pub music_handle: MusicHandle,
}

impl App {

    pub fn new() -> App {
        App {
            browser_items: StatefulList::with_items(App::scan_folder()),
            queue_items: Queue::with_items(vec![]),
            input_mode: InputMode::Browser,
            music_handle: MusicHandle::new(),
        }
    }

    // if item selected is folder, enter folder, else play record.
    pub fn evaluate(&mut self){
        let join = self.selected_item();
        // if folder enter, else play song
        if join.is_dir() {
            env::set_current_dir(join).unwrap();
            self.browser_items = StatefulList::with_items(App::scan_folder());
        } else {
            self.music_handle.play(join);
        }
    }

    // cd into selected directory
    pub fn backpedal(&mut self){
      env::set_current_dir("../").unwrap();
      self.browser_items = StatefulList::with_items(App::scan_folder());
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
            Some(item) => self.music_handle.play(item),
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
        // get absolute path
        let current_dir = env::current_dir().unwrap();
        let join = Path::join(&current_dir, Path::new(&self.browser_items.get_item()));
        join
    }  


    // get files in current directory
    pub fn scan_folder() -> Vec<String>{

        let mut items = Vec::new();
        let options = MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };
        
        for e in glob_with("./*", options).expect("Failed to read glob pattern") {
            match e {
                Ok(item) => {
                    
                    let current_dir = env::current_dir().unwrap();
                    let join = Path::join(&current_dir, Path::new(&item));
                    let ext = Path::new(&item).extension().and_then(OsStr::to_str);       
                
                    // if folder  (Hide Private) enter, else play song
                    if (join.is_dir() && !join.file_name().unwrap().to_str().unwrap().contains(".") ) || (ext.is_some() && 
                    (item.extension().unwrap() == "mp3" || 
                    item.extension().unwrap() == "mp4" || 
                    item.extension().unwrap() == "m4a" || 
                    item.extension().unwrap() == "wav" || 
                    item.extension().unwrap() == "flac" )){
                        items.push(item.to_str().unwrap().to_owned());
                    }         
                },
                Err(_) => (),
            }
        }

        items
    }

}

