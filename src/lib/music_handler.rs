use std::{fs, path::{PathBuf, Path}, thread::{self, JoinHandle, sleep_ms}, sync::{Arc, Mutex}, time::{Duration, Instant, self}, rc::Rc}; 
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

pub struct MusicHandle{
    music_output: Arc<(OutputStream, OutputStreamHandle)>,
    sink: Arc<Sink>,
    song_length: u16,
    time_played: u16,
    pub currently_playing: String,
}


impl MusicHandle {

    pub fn new() -> MusicHandle {
        MusicHandle {
            music_output: Arc::new(OutputStream::try_default().unwrap()),
            sink: Arc::new(Sink::new_idle().0), // more efficient way, shouldnt have to do twice?  
            song_length: 0,
            time_played: 0,
            currently_playing: "CURRENT SONG".to_string()
        }
    }

    // use metadata 
    pub fn get_current_song(&self) -> String { 
        self.currently_playing.clone()
    }

    pub fn get_song_length(&self) -> u16 {
        self.song_length
    }

    pub fn get_time_played(&self) -> u16 {
        self.time_played
    }

    pub fn get_sink_length(&self) -> usize {
        self.sink.len()
    }

    pub fn set_time_played(&mut self, t: u16){
        self.time_played = t;
    }


    // update current song and play
    pub fn play(&mut self, path: PathBuf){
        // if song already playing, need to be able to restart tho
        self.sink.stop();
        self.time_played = 0;
        
        // set currently playing
        self.currently_playing = path.clone().file_name().unwrap().to_str().unwrap().to_string();
        self.song_metadata(&path);

        // reinitialize due to rodio crate
        self.sink = Arc::new(Sink::try_new(&self.music_output.1).unwrap());

        // clone sink for thread
        let sclone = self.sink.clone();


        let _t1 = thread::spawn( move || {
        
            // can send in through function
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();
            sclone.append(source);
            sclone.sleep_until_end();  

        });
    }

    pub fn play_pause(&mut self){
        if self.sink.is_paused(){
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    // clears sink queue, next item auto added
    pub fn skip(&self){
        self.sink.stop();
    }


    pub fn song_metadata(&mut self, path: &PathBuf){
        // trying to access but path has changed
        let f = MediaFileMetadata::new(path).unwrap();
        let dur = f.duration.unwrap();

        // hours, minutes, seconds = vec![&c[..2], &c[3..5], &c[6..8]];
        let m_s: Vec<&str> = vec![&dur[3..5], &dur[6..8]];
        let minutes_to_seconds: u16 = m_s[0].clone().parse::<u16>().expect("couldn't convert time to i32") * 60;
        let seconds: u16 = m_s[1].clone().parse::<u16>().expect("couldn't convert time to i32");
        let song_length = minutes_to_seconds + seconds;
        self.song_length = song_length;
    }

    pub fn increment_time(&mut self){
        if !self.sink.is_paused() && self.sink.len() == 1 {
            self.time_played += 1;
        }
    }

}