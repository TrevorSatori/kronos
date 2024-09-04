use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
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
    command_sender: Sender<Command>,
    command_receiver: Arc<Mutex<Option<Receiver<Command>>>>,
}

#[derive(Debug)]
enum Command {
    Play,
    Pause,
    Stop,
    Seek(i32),
}

// At this point, Player is almost a re-implementation of Sink, with features we need and it lacks.
// It'd probably make more sense to not use sink at all, and just go with `stream_handle.play_raw()`.
// Its `periodicAccess` is also pretty meh. Maybe we can implement our own Source, that doesn't need it?
// Is moving the source between threads every 5ms better than doing atomic operations with Ordering::Relaxed
// on every iteration?
impl Player {
    pub fn new(
        queue: Vec<String>,
        output_stream: &OutputStreamHandle,
    ) -> Self {
        let sink = Arc::new(Sink::try_new(output_stream).unwrap());

        let (command_sender, command_receiver) = channel();

        Self {
            sink,
            queue_items: Arc::new(Queue::new(queue)),
            currently_playing: Arc::new(Mutex::new(None)),
            start_time_bool: Arc::new(AtomicBool::new(false)),
            start_time_u64: Arc::new(AtomicU64::new(0)),
            command_sender,
            command_receiver: Arc::new(Mutex::new(Some(command_receiver))),
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

    pub fn spawn(&self) {
        let sink = self.sink.clone();
        let currently_playing = self.currently_playing.clone();
        let queue_items = self.queue_items.clone();
        let start_time_bool = self.start_time_bool.clone();
        let start_time_u64 = self.start_time_u64.clone();
        let recv = self.command_receiver.clone().lock().unwrap().take().unwrap();

        let set_currently_playing = move |song: Option<Song>| {
            match song.as_ref() {
                Some(song) => {
                    let start_time = song.start_time.clone().unwrap_or(Duration::ZERO);
                    start_time_bool.store(start_time > Duration::ZERO, Ordering::Relaxed);
                    start_time_u64.store(start_time.as_secs(), Ordering::Relaxed);
                }
                None => {
                    start_time_bool.store(false, Ordering::Relaxed);
                    start_time_u64.store(0, Ordering::Relaxed);
                }
            }

            match currently_playing.lock() {
                Ok(mut s) => {
                    debug!("currently_playing = {:?}", song);
                    *s = song;
                }
                Err(err) => {
                    error!("currently_playing.lock() returned an error! {:?}", err);
                    // break;
                }
            };
        };

        thread::Builder::new().name("player".to_string()).spawn(move || {
            loop {
                debug!("queue_items.pop()");
                let song = queue_items.pop();
                debug!("popped {:?}", song.title);

                let path = song.path.clone();
                let start_time = song.start_time.clone().unwrap_or(Duration::ZERO);
                let length = song.length.clone();

                set_currently_playing(Some(song));

                let file = BufReader::new(File::open(path).unwrap());
                let source = Decoder::new(file).unwrap();
                // TODO: can we slice source / implement wrapping iterator, so it ends at song_start + song_length? libs used by rodio consume the entire reader

                debug!("sink.append");
                sink.append(source); // BLOCKING: `sink.append()` does `sleep_until_end()` if it's stopped and it has remaining sounds in it
                // About 5ms could pass before the sink updates its internal status.
                // Until we stop using Sink, this is as good as it gets:
                thread::sleep(Duration::from_millis(15));
                debug!("/sink.append");

                // Songs coming from Cue Sheets are inside one big music file.
                if start_time > Duration::ZERO {
                    sink.try_seek(start_time).unwrap();
                }

                // Start looping until the current song ends OR something wakes us up.
                // When woken up, we check whether we need to immediately exit.
                // If we don't, we recalculate the remaining time until the song ends,
                // and then go back to bed.
                loop {
                    if sink.empty() {
                        break;
                    }

                    let pos = sink.get_pos(); // BUG: sink.get_pos() could return stale data.
                    let true_pos = pos.saturating_sub(start_time);

                    if true_pos >= length {
                        debug!("inner loop: pos >= length");
                        break;
                    }

                    let sleepy_time = length - true_pos;
                    debug!("inner loop: sleepy_time! {:?}", sleepy_time);

                    match recv.recv_timeout(sleepy_time) {
                        Ok(command) => {
                            debug!("command {:?}", command);

                            match command {
                                Command::Play => {}
                                Command::Pause => {}
                                Command::Stop => {
                                    debug!("recv: control_stop");
                                    sink.stop();
                                    break;
                                }
                                Command::Seek(seek) => {
                                    if seek == 0 {
                                        error!("recv: seek == 0");
                                        continue;
                                    }

                                    debug!("recv: seek {}", seek);
                                    let duration = Duration::from_secs(seek.abs() as u64);

                                    let target = if seek > 0 {
                                        pos.saturating_add(duration).min(length + start_time)
                                    } else {
                                        pos.saturating_sub(duration).max(start_time)
                                    };

                                    sink.try_seek(target).unwrap_or_else(|e| {
                                        error!("could not seek {:?}", e);
                                    });
                                }
                            }
                        }
                        Err(_) => {
                            debug!("recv timeout");
                        }
                    }
                }

                // At this point, the sink should be empty and no song should be playing,
                // but Sink updates many of its internal properties only while the song is playing,
                // in its `periodicAccess` that runs every 5ms.
                // `sink.get_pos()` will return incorrect data in this case, for example.
                // This means we may be incorrectly clearing the currently_playing, which
                // would cause the UI to show no playing song at all, even though it'd be actually playing.
                set_currently_playing(None);
            }
        }).unwrap();
    }

    pub fn play_now(&self, song: Song) {
        self.queue_items.add_front(song);

        if self.currently_playing.clone().lock().unwrap().is_some() {
            self.stop();
        }
    }

    pub fn play_now_cue(&self, cue_sheet: CueSheet) {
        let songs = Song::from_cue_sheet(cue_sheet);
        self.queue_items.append(&mut std::collections::VecDeque::from(songs));
        self.stop();
    }

    pub fn toggle(&self) {
        if self.sink.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
        // self.unpark_thread();
        self.command_sender.clone().send(Command::Pause).unwrap()
    }

    pub fn stop(&self) {
        // self.control_stop.store(true, Ordering::SeqCst);
        // self.unpark_thread();
        self.command_sender.clone().send(Command::Stop).unwrap()
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
        // self.control_seek.store(5, Ordering::Relaxed);
        self.command_sender.clone().send(Command::Seek(5)).unwrap()
        // self.unpark_thread();
    }

    pub fn seek_backward(&self) {
        // self.control_seek.store(-5, Ordering::Relaxed);
        self.command_sender.clone().send(Command::Seek(-5)).unwrap()
        // self.unpark_thread();
    }
}
