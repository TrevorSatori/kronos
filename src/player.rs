use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use log::{debug, error, warn};
use rodio::{queue, Decoder, OutputStreamHandle, Sink, Source};

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
    auto_play_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    sink_stop: Arc<AtomicBool>,
}


// At this point, Player is almost a re-implementation of Sink, with features we need and it lacks.
// It'd probably make more sense to not use sink at all.
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
            auto_play_thread: Arc::new(Mutex::new(None)),
            sink_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue_items.clone()
    }

    pub fn get_pos(&self) -> Duration {
        if self.start_time_bool.load(Ordering::Relaxed) {
            let start_time = self.start_time_u64.load(Ordering::Relaxed);
            self.sink.get_pos().saturating_sub(Duration::from_secs(start_time))
        } else {
            self.sink.get_pos()
        }
    }

    pub fn currently_playing(&self) -> Arc<Mutex<Option<Song>>> {
        self.currently_playing.clone()
    }

    // TODO: name that actually describes what this does
    fn queue_changed(&self) {
        let x = self.auto_play_thread.clone();
        let x = x.lock().unwrap();
        let _x = x.as_ref().unwrap().thread().unpark();
    }

    pub fn spawn(&self) {
        let sink = self.sink.clone();
        let currently_playing = self.currently_playing.clone();
        let queue_items = self.queue_items.clone();
        let start_time_bool = self.start_time_bool.clone();
        let start_time_u64 = self.start_time_u64.clone();
        let sink_stop = self.sink_stop.clone();

        let t = thread::Builder::new().name("player".to_string()).spawn(move || {
            loop {
                debug!("will queue_items.pop()");
                let song = queue_items.pop();
                debug!("popped {:?}", song.title);
                let path = song.path.clone();
                let start_time = song.start_time.clone();
                let length = song.length.clone();
                let _song_name = song.title.clone();

                if let Some(start_time) = start_time {
                    start_time_bool.store(true, Ordering::Relaxed);
                    start_time_u64.store(start_time.as_secs(), Ordering::Relaxed);
                } else {
                    start_time_bool.store(false, Ordering::Relaxed);
                }

                debug!("acq currently_playing lock");
                match currently_playing.lock() {
                    Ok(mut s) => {
                        debug!("setting currently_playing to Song {:?}", song);
                        *s = Some(song);
                    }
                    Err(err) => {
                        error!("currently_playing.lock() returned an error! {:?}", err);
                    }
                };
                debug!("currently_playing lock released");

                let file = BufReader::new(File::open(path).unwrap());
                let source = Decoder::new(file).unwrap();

                // TODO: can we slice source / implement wrapping iterator, so it ends at song_start + song_length? libs used by rodio consume the entire reader

                debug!("sink.append");
                sink_stop.store(false, Ordering::SeqCst);
                sink.append(source); // sink.append sleeps if it's stopped + has remaining sounds in it
                debug!("/sink.append");

                if let Some(start_time) = start_time {
                    sink.try_seek(start_time).unwrap();
                }

                debug!("inner loop start");
                loop {
                    debug!("inner loop: get pos");
                    // let pos = if sink.is_paused();
                    let pos = match start_time {
                        Some(start_time) => sink.get_pos().saturating_sub(start_time),
                        _ => sink.get_pos()
                    };
                    debug!("inner loop: pos >= length {:?} {:?}", pos, length);
                    if pos >= length {
                        // TODO: sometimes has old pos after emptying sink!
                        // sink.stop() doesn't atomically sink.position = Duration::ZERO - the internal `periodicAccess` takes care of this
                        // with up to 5ms of delay
                        debug!("inner loop: break pos >= length");
                        break;
                    }
                    debug!("inner loop: sink.empty() {:?}", sink.empty());
                    if sink.empty() {
                        debug!("inner loop: break sink.empty");
                        break;
                    }
                    debug!("inner loop: sink_stop");
                    if sink_stop.load(Ordering::Relaxed) {
                        debug!("inner loop: break sink_stop");
                        sink_stop.store(false, Ordering::SeqCst);
                        break;
                    }
                    debug!("inner loop: park {:?}", length - pos);
                    thread::park_timeout(length - pos);
                    debug!("inner loop: unpark");
                }
                debug!("inner loop end");

                debug!("acq currently_playing lock for clean");
                match currently_playing.lock() {
                    Ok(mut s) => {
                        start_time_bool.store(false, Ordering::Relaxed);
                        *s = None;
                    }
                    Err(err) => {
                        error!("currently_playing.lock() returned an error! {:?}", err);
                    }
                };
                debug!("currently_playing lock for clean released");
            }
        }).unwrap();

        *self.auto_play_thread.clone().lock().unwrap() = Some(t); // ugh
    }

    pub fn play_now(&self, song: Song) {
        // todo: clear current_song
        self.queue_items.add_front(song);
        self.sink.stop();
        self.sink_stop.store(true, Ordering::SeqCst);
        self.queue_changed();
    }

    pub fn play_now_cue(&self, cue_sheet: CueSheet) {
        let songs = Song::from_cue_sheet(cue_sheet);
        self.queue_items.append(&mut std::collections::VecDeque::from(songs));
        self.sink_stop.store(true, Ordering::SeqCst);
        self.queue_changed();
    }

    pub fn toggle(&self) {
        if self.sink.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
        // self.queue_changed();
    }

    pub fn stop(&self) {
        self.sink.stop();
        self.sink_stop.store(true, Ordering::SeqCst);
        self.queue_changed();
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
        self.queue_changed();
    }

    pub fn seek_backward(&self) {
        if let Some(song) = self.currently_playing.lock().unwrap().as_ref() {
            let target = self
                .sink
                .get_pos()
                .saturating_sub(Duration::from_secs(5))
                .max(song.start_time.unwrap_or(Duration::ZERO));
            self.sink.try_seek(target).unwrap_or_else(|e| {
                error!("could not seek {:?}", e);
            });
        }
        self.queue_changed();

    }
}
