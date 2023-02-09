use std::{fs, path::{PathBuf, Path}, thread}; 
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
use rodio::{Sink, Decoder, OutputStream, source::Source};
use std::ffi::OsStr;

use crate::lib::stateful_list::*;
use crate::lib::music::*;

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
    pub queue: Vec<PathBuf>,
    pub current_song: String,
    pub input_mode: InputMode,
    pub music: Music,
    
}


impl App {
    pub fn new() -> App {
        App {
            browser_items: StatefulList::with_items(App::scan_folder()),
            queue_items: StatefulList::with_items(vec![]),
            queue: Vec::new(),
            current_song: "|CURRENT SONG|".to_string(),
            input_mode: InputMode::Browser,
            music: Music::new(),
        }
    }
    

    // if item selected is folder, enter folder, else play record.
    pub fn evaluate(&mut self, selected: usize){

        let join = self.selected_item();
        
        // if folder enter, else play song
        if join.is_dir() {
            env::set_current_dir(join).unwrap();
            self.browser_items = StatefulList::with_items(App::scan_folder());
        } else {
            // self.play_song();
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
                
                    // if folder enter, else play song
                    if join.is_dir() || (ext.is_some() && (item.extension().unwrap() == "mp3")){
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

    pub fn enqueu(&mut self, song: PathBuf){

        // push songs to queue as Pathbuf and stateful list as string
        self.queue.push(song.clone());
        self.queue_items = StatefulList::with_items(vec![song.file_name().unwrap().to_str().unwrap().to_string().clone()]);
    
    }

    pub fn get_current_song(&self) -> String{
        self.current_song.clone()
    }

    // update current song and play
    pub fn play(&mut self, path: PathBuf){
        
        self.current_song = path.file_name().unwrap().to_str().unwrap().to_string().clone();

        let t1 = thread::spawn(||{
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).expect("Couldn't create sink");
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();
            
            sink.append(source);
            sink.sleep_until_end();
            // self.sink.play();
        });
    }

}
