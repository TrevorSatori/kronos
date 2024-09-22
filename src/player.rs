use std::{
    sync::{
        Arc,
        Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{channel, Receiver, RecvTimeoutError, Sender},
    },
    thread,
    time::Duration,
};

use log::{debug, error};
use rodio::{Decoder, OutputStreamHandle, Source};

use crate::{
    cue::CueSheet,
    structs::{Queue, Song},
    sample::{create_source_from_file, FullSource},
};

pub struct Player {
    output_stream: OutputStreamHandle,
    queue_items: Arc<Queue>,
    currently_playing: Arc<Mutex<Option<Song>>>,
    currently_playing_start_time: Arc<AtomicU64>,
    command_sender: Sender<Command>,
    command_receiver: Arc<Mutex<Option<Receiver<Command>>>>,
    is_stopped: Arc<AtomicBool>,
    volume: Arc<Mutex<f32>>,
    pause: Arc<AtomicBool>,
    position: Arc<Mutex<Duration>>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Command {
    Play,
    Pause,
    Stop,
    Seek(i32),
}

impl Player {
    pub fn new(queue: Vec<Song>, output_stream: OutputStreamHandle) -> Self {
        let (command_sender, command_receiver) = channel();

        Self {
            output_stream,
            queue_items: Arc::new(Queue::new(queue)),
            currently_playing: Arc::new(Mutex::new(None)),
            currently_playing_start_time: Arc::new(AtomicU64::new(0)),
            command_sender,
            command_receiver: Arc::new(Mutex::new(Some(command_receiver))),
            is_stopped: Arc::new(AtomicBool::new(true)),
            volume: Arc::new(Mutex::new(1.0)),
            pause: Arc::new(AtomicBool::new(false)),
            position: Arc::new(Mutex::new(Duration::ZERO)),
        }
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue_items.clone()
    }

    pub fn get_pos(&self) -> Duration {
        let start_time = self.currently_playing_start_time.load(Ordering::Relaxed);
        let pos = self.position.lock().unwrap();
        pos.saturating_sub(Duration::from_secs(start_time))
    }

    pub fn currently_playing(&self) -> Arc<Mutex<Option<Song>>> {
        self.currently_playing.clone()
    }

    pub fn spawn(&self) {
        let output_stream = self.output_stream.clone();
        let recv = self.command_receiver.clone().lock().unwrap().take().unwrap();
        let queue_items = self.queue_items.clone();
        let currently_playing = self.currently_playing.clone();
        let song_start_time = self.currently_playing_start_time.clone();

        let position = self.position.clone();
        let volume = self.volume.clone();
        let pause = self.pause.clone();

        let is_stopped = self.is_stopped.clone();
        let must_stop = Arc::new(AtomicBool::new(false));
        let (ended_sender, ender_recv) = channel::<()>();
        let must_seek = Arc::new(Mutex::new(None));

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
                // Grab the next song in the queue. If there isn't one, we block until one comes in.
                debug!("queue_items.pop()...");
                let song = queue_items.pop();
                is_stopped.store(false, Ordering::SeqCst);
                debug!("queue_items.pop() -> popped {:?}", song.title);

                let path = song.path.clone();
                let start_time = song.start_time.clone();
                let length = song.length.clone();

                set_currently_playing(Some(song));

                let periodic_access = {
                    let is_stopped = is_stopped.clone();
                    let must_stop = must_stop.clone();
                    let ended_sender = ended_sender.clone();
                    let position = position.clone();
                    let volume = volume.clone();
                    let pause = pause.clone();
                    let must_seek = must_seek.clone();

                    move |src: &mut FullSource| {
                        if must_stop.swap(false, Ordering::SeqCst) {
                            src.stop();
                            src.inner_mut().skip();
                            *position.lock().unwrap() = Duration::ZERO;
                            is_stopped.store(true, Ordering::SeqCst);
                            let _ = ended_sender.send(());
                        } else {
                            *position.lock().unwrap() = src.inner().inner().inner().inner().get_pos();
                        }

                        let amp = src.inner_mut().inner_mut();
                        amp.set_factor(*volume.lock().unwrap());

                        let pausable = amp.inner_mut();
                        pausable.set_paused(pause.load(Ordering::SeqCst));

                        if let Some(seek) = must_seek.lock().unwrap().take() {
                            if let Err(err) = amp.try_seek(seek) {
                                error!("start_time > 0 try_seek() error. {:?}", err)
                            }
                        }
                    }
                };

                let mut source = create_source_from_file(path, periodic_access);

                if start_time > Duration::ZERO {
                    debug!("start_time > Duration::ZERO, {:?}", start_time);
                    if let Err(err) = source.inner_mut().inner_mut().try_seek(start_time) {
                        error!("start_time > 0 try_seek() error. {:?}", err)
                    }
                    *position.lock().unwrap() = start_time;
                }

                debug!("s.play_raw()");
                if let Err(err) = output_stream.play_raw(source) {
                    error!("os.play_raw error! {:?}", err);
                    continue;
                }

                // Start looping until the current song ends OR something wakes us up.
                // When woken up, we check whether we need to immediately exit.
                // If we don't, we recalculate the remaining time until the song ends,
                // and then go back to bed.
                loop {
                    let sleepy_time = if pause.load(Ordering::SeqCst) {
                        Duration::MAX
                    } else {
                        let abs_pos = position.lock().unwrap().saturating_sub(start_time);
                        if abs_pos >= length {
                            debug!("inner loop: pos >= length, {:?} > {:?}", abs_pos, length);
                            break;
                        }
                        length - abs_pos
                    };

                    // debug!("inner loop: sleepy_time! {:?}", sleepy_time);

                    match recv.recv_timeout(sleepy_time) {
                        Ok(command) => {
                            // debug!("{:?}", command);
                            match command {
                                Command::Play => {
                                    pause.store(false, Ordering::SeqCst);
                                }
                                Command::Pause => {
                                    pause.store(true, Ordering::SeqCst);
                                }
                                Command::Stop => {
                                    break;
                                }
                                Command::Seek(seek) => {
                                    // NOTE: "intense" seek causes `ALSA lib pcm.c:8740:(snd_pcm_recover) underrun occurred`.
                                    // See https://github.com/RustAudio/cpal/pull/909

                                    if seek == 0 {
                                        error!("Command::Seek(0)");
                                        continue;
                                    }

                                    if is_stopped.load(Ordering::SeqCst) || must_stop.load(Ordering::SeqCst) {
                                        continue;
                                    }

                                    let seek_abs = Duration::from_secs(seek.abs() as u64);
                                    let pos = position.lock().unwrap();

                                    let target = if seek > 0 {
                                        pos.saturating_add(seek_abs)
                                    } else {
                                        pos.saturating_sub(seek_abs).max(start_time)
                                    };

                                    // If we'd seek past song end, skip seeking and just move to next song instead.
                                    if target > length + start_time {
                                        debug!("Seeking past end");
                                        break;
                                    }

                                    debug!("Seek({:?})", target);
                                    *must_seek.lock().unwrap() = Some(target);

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
                must_stop.store(true, Ordering::SeqCst);
                if let Err(err) = ender_recv.recv() {
                    error!("ender_recv.recv {:?}", err);
                }
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
        if self.pause.load(Ordering::SeqCst) {
            self.command_sender.send(Command::Play).unwrap();
        } else {
            self.command_sender.send(Command::Pause).unwrap();
        }
    }

    pub fn stop(&self) {
        self.command_sender.send(Command::Stop).unwrap()
    }

    pub fn seek(&self, seek: i32) {
        // Avoid queueing seek commands if nothing is playing
        if self.is_stopped.load(Ordering::SeqCst) {
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
        let mut volume = *self.volume.lock().unwrap() + amount;
        if volume < 0. {
            volume = 0.;
        } else if volume > 1. {
            volume = 1.;
        }
        *self.volume.lock().unwrap() = volume;
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        let _ = self.command_sender.send(Command::Stop);
    }
}
