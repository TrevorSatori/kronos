use std::{fs, path::{PathBuf, Path}, thread, sync::Arc}; 
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
    pub current_song: String,
    pub input_mode: InputMode,
    
}


// let (_stream, stream_handle) =
impl App {
    pub fn new() -> App {
        App {
            browser_items: StatefulList::with_items(App::scan_folder()),
            queue_items: StatefulList::with_items(vec![]),
            current_song: "|CURRENT SONG|".to_string(),
            input_mode: InputMode::Browser,
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
        let join = Path::join(&current_dir, Path::new(&self.browser_items.items[self.browser_items.curr]));
        join
    }  

    pub fn get_current_song(&self) -> String{
        self.current_song.clone()
    }

    // update current song and play
    pub fn play(&mut self, path: PathBuf){
        
        self.current_song = path.file_name().unwrap().to_str().unwrap().to_string();
        
        
        let t1 = thread::spawn(||{
        
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).expect("Couldn't create sink");
            sink.append(source);
            sink.sleep_until_end();
        });
    }
}


// TODO, use same thread for queue songs
// TODO, get length of song from file