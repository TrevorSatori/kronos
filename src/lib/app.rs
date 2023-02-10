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


// app is responsible for handling state //
// keeps track of which Field you are in (QUEUE, Browser)
// updates and handles list state

pub enum InputMode {
    Browser,
    Queue,
}


pub struct App {
    pub browser_items: StatefulList<String>,
    pub queue_items: StatefulList<String>,
    pub currently_playing: String,
    pub input_mode: InputMode,
    music_output: Arc<(OutputStream, OutputStreamHandle)>,
    sink: Arc<Sink>,

    time: u16,
}

impl App {

    pub fn new() -> App {

        // let sink = Sink::try_new(&stream_handle).expect("Couldn't create sink");
        App {
            browser_items: StatefulList::with_items(App::scan_folder()),
            queue_items: StatefulList::with_items(vec![]),
            currently_playing: "|CURRENT SONG|".to_string(),
            input_mode: InputMode::Browser,
            music_output: Arc::new(OutputStream::try_default().unwrap()),
            sink: Arc::new(Sink::new_idle().0), // more efficient way, shouldnt have to do twice?
            time: 0,
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
            self.play(join);
        }
    }

    // cd into selected directory
    pub fn backpedal(&mut self){
      env::set_current_dir("../").unwrap();
      self.browser_items = StatefulList::with_items(App::scan_folder());
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
                    if (join.is_dir() && !join.file_name().unwrap().to_str().unwrap().contains(".") ) || (ext.is_some() && (item.extension().unwrap() == "mp3" || item.extension().unwrap() == "mp4" || item.extension().unwrap() == "m4a" || item.extension().unwrap() == "wav")){
                        items.push(item.to_str().unwrap().to_owned());
                    }         
                },
                Err(_) => (),
            }
        }

        items
    }

    // get file path
    pub fn selected_item(&self) -> PathBuf{
        // get absolute path
        let current_dir = env::current_dir().unwrap();
        let join = Path::join(&current_dir, Path::new(&self.browser_items.get_item()));
        join
    }  

    pub fn get_current_song(&self) -> String{
        self.currently_playing.clone()
    }

    // update current song and play
    pub fn play(&mut self, path: PathBuf){
        // if song already playing, need to be able to restart tho
        if self.sink.len() == 1 {
            return;
        } else {

            // set currently playing
            self.currently_playing = path.file_name().unwrap().to_str().unwrap().to_string();
            // reinitialize due to rodio crate
            self.sink = Arc::new(Sink::try_new(&self.music_output.1).unwrap());

            //clone sink for thread
            let sclone = self.sink.clone();

            let _t1 = thread::spawn( move || {
            
                // can send in through function
                let file = BufReader::new(File::open(path).unwrap());
                let source = Decoder::new(file).unwrap();
                sclone.append(source);
                sclone.sleep_until_end();
                
    
            });
        };
    }

    pub fn play_pause(&mut self){

        if self.sink.is_paused(){
            self.sink.play()
        } else {
            self.sink.pause()
        }
        
    }


    // track song progress, NEEDS TIME RUNNING
    pub fn song_progress(&mut self) -> u16 {

        // edge case
        if self.sink.len() != 1 {
            return 0
        } else {

            let f = MediaFileMetadata::new(&self.currently_playing).unwrap();
            let dur = f.duration.unwrap();

            // hours, minutes, seconds = vec![&c[..2], &c[3..5], &c[6..8]];
            let m_s: Vec<&str> = vec![&dur[3..5], &dur[6..8]];
            let minutes_to_seconds: u16 = m_s[0].clone().parse::<u16>().expect("couldn't convert time to i32") * 60;
            let seconds: u16 = m_s[1].clone().parse::<u16>().expect("couldn't convert time to i32");
            let song_length = minutes_to_seconds + seconds;
            


            if !self.sink.is_paused(){
                self.time += 1;
            }
            
            ((self.time - 0) * 100) / song_length
        }

        
    }

}


// TODO, use same thread for queue songs
// TODO, Gauge progress
// TODO, function for song name or file name
// When song is done, pop item out of queue and start new 

// need to spawn thread for music, but also be able to have access throughout life of app
// function can be ruled out, must be part of struct.

// WHAT NEEDS TO BE ACCESSED AND SENT TO THREAD?
// ACCESSED: song play time
// Sent: Pause, play, next

/*
    time = time now

    if pause, save time passed to variable

*/
