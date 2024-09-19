use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use log::{debug, error, warn};
use rodio::{Decoder, OutputStreamHandle, Sink};

use crate::{
    cue::CueSheet,
    structs::{Queue, Song},
};

pub struct Player {
    sink: Arc<Sink>,
    currently_playing: Arc<Mutex<Option<Song>>>,
    queue_items: Arc<Queue>,
    song_start_time: Arc<AtomicU64>, // We duplicate this off of song to save us a song.lock(). Relaxed ordering compiles down to the same ASM a normal u64. TODO: maybe make Song thread-friendly, so we won't need the Mutex.
    command_sender: Sender<Command>,
    command_receiver: Arc<Mutex<Option<Receiver<Command>>>>,
}

#[derive(Debug)]
#[allow(dead_code)]
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
    pub fn new(queue: Vec<Song>, output_stream: &OutputStreamHandle) -> Self {
        let sink = Arc::new(Sink::try_new(output_stream).unwrap());

        let (command_sender, command_receiver) = channel();

        Self {
            sink,
            queue_items: Arc::new(Queue::new(queue)),
            currently_playing: Arc::new(Mutex::new(None)),
            song_start_time: Arc::new(AtomicU64::new(0)),
            command_sender,
            command_receiver: Arc::new(Mutex::new(Some(command_receiver))),
        }
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue_items.clone()
    }

    pub fn get_pos(&self) -> Duration {
        let start_time = self.song_start_time.load(Ordering::Relaxed);
        self.sink.get_pos().saturating_sub(Duration::from_secs(start_time))
    }

    pub fn currently_playing(&self) -> Arc<Mutex<Option<Song>>> {
        self.currently_playing.clone()
    }

    pub fn spawn(&self) {
        let sink = self.sink.clone();
        let currently_playing = self.currently_playing.clone();
        let queue_items = self.queue_items.clone();
        let recv = self.command_receiver.clone().lock().unwrap().take().unwrap();

        let song_start_time = self.song_start_time.clone();
        let set_currently_playing = move |song: Option<Song>| {
            let start_time = song
                .as_ref()
                .and_then(|song| Some(song.start_time))
                .unwrap_or(Duration::ZERO)
                .as_secs();
            song_start_time.store(start_time, Ordering::Relaxed);

            match currently_playing.lock() {
                Ok(mut s) => {
                    debug!("currently_playing = {:?}", song);
                    *s = song;
                }
                Err(err) => {
                    error!("currently_playing.lock() returned an error! {:?}", err);
                }
            };
        };

        thread::Builder::new().name("player".to_string()).spawn(move || {
            loop {
                debug!("queue_items.pop()");
                let song = queue_items.pop();
                debug!("popped {:?}", song.title);

                let path = song.path.clone();
                let start_time = song.start_time.clone();
                let length = song.length.clone();

                set_currently_playing(Some(song));

                let file = BufReader::new(File::open(path).unwrap());
                let source = Decoder::new(file).unwrap();
                // TODO: can we slice source / implement wrapping iterator, so it ends at song_start + song_length? libs used by rodio consume the entire reader

                debug!("sink.append()");
                sink.append(source); // BLOCKING: `sink.append()` does `sleep_until_end()` if it's stopped and it has remaining sounds in it
                // About 5ms could pass before the sink updates its internal status.
                // Until we stop using Sink, this is as good as it gets:
                thread::sleep(Duration::from_millis(15));
                debug!("sink.appended. sink.get_pos()={:?}", sink.get_pos());

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
                        debug!("sink.empty(). breaking out of inner loop.");
                        break;
                    }

                    let sleepy_time = if sink.is_paused() {
                        Duration::MAX
                    } else {
                        let true_pos = sink.get_pos().saturating_sub(start_time); // BUG: sink.get_pos() could return stale data.
                        if true_pos >= length {
                            debug!("inner loop: pos >= length, {:?} > {:?}", true_pos, length);
                            break;
                        }
                        length - true_pos
                    };

                    debug!("inner loop: sleepy_time! {:?}", sleepy_time);

                    match recv.recv_timeout(sleepy_time) {
                        Ok(command) => {
                            debug!("{:?}", command);
                            match command {
                                Command::Play => {

                                }
                                Command::Pause => {

                                }
                                Command::Stop => {
                                    sink.stop();
                                    break;
                                }
                                Command::Seek(seek) => {
                                    // TODO: "intense" seek causes `ALSA lib pcm.c:8740:(snd_pcm_recover) underrun occurred`.
                                    // See https://github.com/RustAudio/cpal/pull/909
                                    if seek == 0 {
                                        error!("Command::Seek(0)");
                                        continue;
                                    }

                                    let seek_abs = Duration::from_secs(seek.abs() as u64);
                                    let pos = sink.get_pos();

                                    let target = if seek > 0 {
                                        pos.saturating_add(seek_abs).min(length + start_time)
                                    } else {
                                        pos.saturating_sub(seek_abs).max(start_time)
                                    };

                                    sink.try_seek(target).unwrap_or_else(|e| {
                                        error!("could not seek {:?}", e);
                                    });
                                }
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            debug!("RecvTimeoutError::Timeout");
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            warn!("RecvTimeoutError::Disconnected");
                            return;
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

    pub fn play_song(&self, song: Song) {
        self.queue_items.add_front(song);

        if self.currently_playing.clone().lock().unwrap().is_some() {
            self.stop();
        }
    }

    pub fn enqueue_song(&self, song: Song) {
        self.queue_items.add_back(song);
    }

    pub fn enqueue_cue(&self, cue_sheet: CueSheet) {
        let songs = Song::from_cue_sheet(cue_sheet);
        self.queue_items.append(&mut std::collections::VecDeque::from(songs));
    }

    pub fn toggle(&self) {
        if self.sink.is_paused() {
            self.sink.play();
            self.command_sender.send(Command::Play).unwrap();
        } else {
            self.sink.pause();
            self.command_sender.send(Command::Pause).unwrap();
        }
    }

    pub fn stop(&self) {
        self.command_sender.send(Command::Stop).unwrap()
    }

    pub fn seek(&self, seek: i32) {
        self.command_sender.send(Command::Seek(seek)).unwrap()
    }

    pub fn seek_forward(&self) {
        self.seek(5);
    }

    pub fn seek_backward(&self) {
        self.seek(-5);
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
}
