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
    song_finished_tx: Sender<()>,
    song_finished_rx: Arc<Mutex<Receiver<()>>>, // std::sync::RwLock?
    auto_play_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Player {
    pub fn new(
        queue: Vec<String>,
        output_stream: &OutputStreamHandle,
    ) -> Self {
        let sink = Arc::new(Sink::try_new(output_stream).unwrap());

        let (tx, rx): (Sender<()>, Receiver<()>) = channel();

        Self {
            sink,
            queue_items: Arc::new(Queue::new(queue)),
            currently_playing: Arc::new(Mutex::new(None)),
            start_time_bool: Arc::new(AtomicBool::new(false)),
            start_time_u64: Arc::new(AtomicU64::new(0)),
            song_finished_tx: tx,
            song_finished_rx: Arc::new(Mutex::new(rx)),
            auto_play_thread: Arc::new(Mutex::new(None))
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

    fn send_song_finished(&self) {
        if self.start_time_bool.load(Ordering::Relaxed) {
            self.song_finished_tx.clone().send(()).unwrap_or_else(|e| error!("Could not send song_finished_tx {:?}", e));
        }
        let x = self.auto_play_thread.clone();
        let x = x.lock().unwrap();
        let x = x.as_ref().unwrap().thread().unpark();
    }

    pub fn spawn(&self) {
        self.spawn_player_thread();
        self.spawn_auto_play_thread();
    }

    fn spawn_player_thread(&self) {
        let sink = self.sink.clone();
        let currently_playing = self.currently_playing.clone();
        let queue_items = self.queue_items.clone();
        let start_time_bool = self.start_time_bool.clone();
        let start_time_u64 = self.start_time_u64.clone();
        let rx = self.song_finished_rx.clone();

        thread::spawn(move || {
            loop {
                let song = queue_items.pop();
                let path = song.path.clone();
                let start_time = song.start_time.clone();
                let song_length = song.length;

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
                // TODO: can we slice source / implement wrapping iterator, so it ends at song_start + song_length? libs used by rodio consume the entire reader

                sink.append(source);

                if let Some(start_time) = start_time {
                    sink.try_seek(start_time).unwrap();
                // } else {
                //     sink.sleep_until_end();
                }

                // Wait until current song has finished playing
                let rx = rx.lock().unwrap();
                rx.recv().unwrap_or_else(|e| error!("sink.try_seek error in player thread {:?}", e));

                match currently_playing.lock() {
                    Ok(mut s) => {
                        start_time_bool.store(false, Ordering::Relaxed);
                        *s = None;
                    }
                    Err(err) => {
                        error!("spawn_player_thread: currently_playing.lock() returned an error! {:?}", err);
                    }
                };
            }
        });
    }

    fn spawn_auto_play_thread(&self) {
        let sink = self.sink.clone();
        let currently_playing = self.currently_playing.clone();
        let tx = self.song_finished_tx.clone();

        // TODO: gapless playing???
        //   maybe: sleep_until_song_end + sender.send on queue add?
        //   std::sync::Condvar.wait_while? thread::park_timeout?

        let t = thread::spawn(move || loop {
            let is_cue_song_playing = match currently_playing.lock() {
                Ok(song) => {
                    if let Some(song) = song.as_ref() {
                        if let Some(song_start_time) = song.start_time {
                            let pos = sink.get_pos().saturating_sub(song_start_time);

                            if pos > song.length {
                                if let Err(err) = tx.send(()) {
                                    error!("Error sending song_ended! Will break out of loop, just in case. {:#?}", err);
                                    break;
                                }
                            }

                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                Err(err) => {
                    error!("spawn_player_thread: currently_playing.lock() returned an error! {:?}", err);
                    break;
                }
            };
            if is_cue_song_playing || sink.is_paused() || sink.empty() {
                debug!("thread::park_timeout");
                // while pos < len {
                    thread::park_timeout(Duration::from_millis(2000));
                // }
                debug!("thread::unparked");
            } else {
                debug!("sink.sleep_until_end");
                sink.sleep_until_end();
                if let Err(err) = tx.send(()) {
                    error!("Error sending song_ended after sink.sleep_until_end! Will break out of loop, just in case. {:#?}", err);
                    break;
                }
                debug!("sink.slept");
            }
        });

        *self.auto_play_thread.clone().lock().unwrap() = Some(t); // ugh
    }

    pub fn play_now(&self, song: Song) {
        self.send_song_finished();
        self.queue_items.add_front(song);
        self.sink.stop();
    }

    pub fn play_now_cue(&self, cue_sheet: CueSheet) {
        self.send_song_finished();
        let songs = Song::from_cue_sheet(cue_sheet);
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
        self.send_song_finished();
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

    }
}
