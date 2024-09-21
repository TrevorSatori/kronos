use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
    is_stopped: Arc<AtomicBool>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Command {
    Play,
    Pause,
    Stop,
    Seek(i32),
}

// See TODO.md to understand some of the unintuitive, weird things Player does (like seemingly random `thread::sleep`s).
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
            is_stopped: Arc::new(AtomicBool::new(true)),
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
        let is_stopped = self.is_stopped.clone();

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
                // Grab the next song in the queue. If there isn't one, we block until one comes in.
                let song = queue_items.pop();
                debug!("popped {:?}", song.title);

                let path = song.path.clone();
                let start_time = song.start_time.clone();
                let length = song.length.clone();

                set_currently_playing(Some(song));

                let file = BufReader::new(File::open(path).unwrap());
                let source = Decoder::new(file).unwrap();

                debug!("sink.append(), {}, get_pos={:?}", sink.len(), sink.get_pos());
                sink.append(source); // Would blocking if sink.len() > 0, but we always have up to 1 source in the sink at any given time.
                thread::sleep(Duration::from_millis(15));
                debug!("sink.appended. sink.get_pos()={:?}, len={}", sink.get_pos(), sink.len());
                is_stopped.store(false, Ordering::SeqCst);
                sink.play();

                // Songs coming from Cue Sheets are inside one big music file.
                if start_time > Duration::ZERO {
                    debug!("start_time > Duration::ZERO. start_time={:?}, sink.get_pos()={:?}", start_time, sink.get_pos());
                    sink.try_seek(start_time).unwrap();
                    thread::sleep(Duration::from_millis(15));
                    debug!("start_time > Duration::ZERO. start_time={:?}, sink.get_pos() after try_seek ={:?}", start_time, sink.get_pos());
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
                        let true_pos = sink.get_pos().saturating_sub(start_time);
                        if true_pos >= length {
                            debug!("inner loop: pos >= length, {:?} > {:?}; sink.empty()={}", true_pos, length, sink.empty());
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
                                    break;
                                }
                                Command::Seek(seek) => {
                                    if sink.empty() || is_stopped.load(Ordering::SeqCst) {
                                        continue;
                                    }

                                    // TODO: "intense" seek causes `ALSA lib pcm.c:8740:(snd_pcm_recover) underrun occurred`.
                                    // See https://github.com/RustAudio/cpal/pull/909

                                    if seek == 0 {
                                        error!("Command::Seek(0)");
                                        continue;
                                    }

                                    let seek_abs = Duration::from_secs(seek.abs() as u64);
                                    let pos = sink.get_pos();

                                    let target = if seek > 0 {
                                        pos.saturating_add(seek_abs)
                                    } else {
                                        pos.saturating_sub(seek_abs).max(start_time)
                                    };

                                    // If we'd seek past song end, skip seeking and just move to next song instead.
                                    if target > length + start_time {
                                        break;
                                    }

                                    debug!("Seek({:?})", target);
                                    sink.try_seek(target).unwrap();
                                }
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            // Playing song reached its end. We want to move on to the next song.
                            debug!("RecvTimeoutError::Timeout");
                            break;
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            // Most of the time, not a real error. This happens because the command_sender was dropped,
                            // which happens when the player itself was dropped, so we just want to exit.
                            debug!("RecvTimeoutError::Disconnected");
                            return;
                        }
                    }
                }

                set_currently_playing(None);
                is_stopped.store(true, Ordering::SeqCst);
                sink.clear();
                thread::sleep(Duration::from_millis(15));
            }
        }).unwrap();
    }

    pub fn play_song(&self, song: Song) {
        self.queue_items.add_front(song);

        if self.currently_playing.lock().unwrap().is_some() {
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
        if self.is_stopped.load(Ordering::SeqCst) {
            return;
        }
        if self.sink.is_paused() {
            self.sink.play();
            self.command_sender.send(Command::Play).unwrap();
        } else {
            self.sink.pause();
            self.command_sender.send(Command::Pause).unwrap();
        }
    }

    pub fn stop(&self) {
        // Avoid queueing stop commands
        if self.is_stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        self.command_sender.send(Command::Stop).unwrap()
    }

    pub fn seek(&self, seek: i32) {
        // Avoid queueing seek commands if nothing is playing
        if self.is_stopped.load(Ordering::SeqCst) || self.sink.len() == 0 {
            return;
        }
        // Note: Symphonia seems to be the only decoder that supports seeking in Rodio (that we really care about), but it can fail.
        // Rodio's `Source for TrackPosition` does have its own `try_seek`, though, as well as `Source for SamplesBuffer`.
        // Are we using those (indirectly), or just Symphonia?
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
