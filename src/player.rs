use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use log::{debug, error, warn};
use rodio::{Decoder, OutputStreamHandle, Sink};

use crate::{
    structs::{
        song::{Song},
        queue::Queue,
    },
    cue::CueSheet,
};

pub struct Player {
    sink: Arc<Sink>,
    currently_playing: Arc<Mutex<Option<Song>>>,
    queue_items: Arc<Queue>,
    start_time_bool: Arc<AtomicBool>,
    start_time_u64: Arc<AtomicU64>,
}

impl Player {
    pub fn new(
        queue: Vec<String>,
        output_stream: &OutputStreamHandle,
    ) -> Self {
        let sink = Arc::new(Sink::try_new(output_stream).unwrap());

        Self {
            sink,
            queue_items: Arc::new(Queue::new(queue)),
            currently_playing: Arc::new(Mutex::new(None)),
            start_time_bool: Arc::new(AtomicBool::new(false)),
            start_time_u64: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue_items.clone()
    }

    pub fn get_pos(&self) -> Duration {
        if self.start_time_bool.load(Ordering::Relaxed) {
            let start_time = self.start_time_u64.load(Ordering::Relaxed);
            // debug!("hmm {:?}", Duration::from_secs(start_time));
            // let x = self.sink.get_pos();
            // debug!("xxxx {:?}", x);
            // let x = x.saturating_sub(Duration::from_secs(start_time));
            // debug!("hmm {start_time} {:?}", x);
            // x
            self.sink.get_pos().saturating_sub(Duration::from_secs(start_time))
        } else {
            self.sink.get_pos()
        }
    }

    pub fn currently_playing(&self) -> Arc<Mutex<Option<Song>>> {
        self.currently_playing.clone()
    }

    pub fn spawn_player_thread(&self) {
        let sink = self.sink.clone();
        let currently_playing = self.currently_playing.clone();
        let queue_items = self.queue_items.clone();
        let start_time_bool = self.start_time_bool.clone();
        let start_time_u64 = self.start_time_u64.clone();

        thread::spawn(move || {
            loop {
                let song = queue_items.pop();
                let path = song.path.clone();
                let start_time = song.start_time.clone();

                if let Some(start_time) = start_time {
                    start_time_bool.store(true, Ordering::Relaxed);
                    start_time_u64.store(start_time.as_secs(), Ordering::Relaxed);
                } else {
                    start_time_bool.store(false, Ordering::Relaxed);
                }

                match currently_playing.lock() {
                    Ok(mut s) => {
                        *s = Some(song);
                    }
                    Err(err) => {
                        error!("spawn_player_thread: currently_playing.lock() returned an error! {:?}", err);
                    }
                };

                let file = BufReader::new(File::open(path).unwrap());
                let source = Decoder::new(file).unwrap();

                sink.append(source);

                if let Some(start_time) = start_time {
                    sink.try_seek(start_time).unwrap();
                    // TODO: thread::sleep(song.length)
                }

                sink.sleep_until_end(); // TODO: incorrect for CueSheets. thread::sleep(song.length)

                match currently_playing.lock() {
                    Ok(mut s) => {
                        *s = None;
                    }
                    Err(err) => {
                        error!("spawn_player_thread: currently_playing.lock() returned an error! {:?}", err);
                    }
                };
            }
        });
    }

    pub fn play_now(&self, song: Song) {
        self.queue_items.add_front(song);
        self.sink.stop();
    }

    pub fn play_now_cue(&self, cue_sheet: CueSheet) {
        let songs = Song::from_cue_sheet(cue_sheet);
        debug!("play_now_cue songs= {:#?}", songs);
        self.queue_items.append(&mut std::collections::VecDeque::from(songs));
    }

    pub fn toggle(&self) {
        if self.sink.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn change_volume(&self, amount: f32) {
        let mut volume = self.sink.volume() + amount;
        if volume < 0. {
            volume = 0.;
        } else if volume > 1. {
            volume = 1.;
        }
        self.sink.set_volume(volume)
    }

    pub fn seek_forward(&self) {
        if let Some(song) = self.currently_playing.lock().unwrap().as_ref() {
            let target = self
                .sink
                .get_pos()
                .saturating_add(Duration::from_secs(5))
                .min(song.length + song.start_time.unwrap_or(Duration::ZERO));
            self.sink.try_seek(target).unwrap_or_else(|e| {
                error!("could not seek {:?}", e);
            });
        }
    }

    pub fn seek_backward(&self) {
        let target = self
            .sink
            .get_pos()
            .saturating_sub(Duration::from_secs(5))
            .max(Duration::from_secs(0));
        self.sink.try_seek(target).unwrap_or_else(|e| {
            error!("could not seek {:?}", e);
        });
    }
}
