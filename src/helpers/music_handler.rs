use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use lofty::{AudioFile, Probe};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use super::gen_funcs;

pub struct MusicHandle {
    music_output: Arc<(OutputStream, OutputStreamHandle)>,
    sink: Arc<Sink>,
    song_length: u16,
    time_played: Arc<Mutex<u16>>,
    currently_playing: String,
    volume: f32,
}

impl Default for MusicHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicHandle {
    pub fn new() -> Self {
        Self {
            music_output: Arc::new(OutputStream::try_default().unwrap()),
            sink: Arc::new(Sink::new_idle().0), // more efficient way, shouldnt have to do twice?
            song_length: 0,
            time_played: Arc::new(Mutex::new(0)),
            currently_playing: "CURRENT SONG".to_string(),
            volume: 1.,
        }
    }

    pub fn currently_playing(&self) -> String {
        self.currently_playing.clone()
    }

    pub fn song_length(&self) -> u16 {
        self.song_length
    }

    pub fn time_played(&self) -> u16 {
        *self.time_played.lock().unwrap()
    }

    pub fn sink_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn set_time_played(&mut self, t: u16) {
        *self.time_played.lock().unwrap() = t;
    }
    // set currently playing song
    pub fn set_currently_playing(&mut self, path: &PathBuf) {
        self.currently_playing = gen_funcs::audio_display(path);
    }

    // update current song and play
    pub fn play(&mut self, path: PathBuf) {
        // if song already playing, need to be able to restart tho
        self.sink.stop();
        *self.time_played.lock().unwrap() = 0;

        // set currently playing
        self.currently_playing = path
            .clone()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        self.set_currently_playing(&path);
        self.update_song_length(&path);

        // reinitialize due to rodio crate
        self.sink = Arc::new(Sink::try_new(&self.music_output.1).unwrap());

        // clone sink for thread
        let sclone = self.sink.clone();

        let tpclone = self.time_played.clone();

        let _t1 = thread::spawn(move || {
            // can send in through function
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();

            // Arc inside a thread inside a thread. BOOM, INCEPTION
            let sink_clone_2 = sclone.clone();
            let tpclone2 = tpclone.clone();

            sclone.append(source);

            let _ = thread::spawn(move || {
                // sleep for 1 second then increment count
                while sink_clone_2.len() == 1 {
                    thread::sleep(Duration::from_secs(1));

                    if !sink_clone_2.is_paused() {
                        *tpclone2.lock().unwrap() += 1;
                    }
                }
            });
            // if sink.stop, thread destroyed.
            sclone.sleep_until_end();
        });
    }

    pub fn play_pause(&mut self) {
        if self.sink.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    pub fn skip(&self) {
        self.sink.stop();
    }

    /// Update `self.song_length` with the provided file.
    pub fn update_song_length(&mut self, path: &PathBuf) {
        let path = Path::new(&path);
        let tagged_file = Probe::open(path)
            .expect("ERROR: Bad path provided!")
            .read()
            .expect("ERROR: Failed to read file!");

        let properties = &tagged_file.properties();
        let duration = properties.duration();

        // update song length, currently playing
        self.song_length = duration.as_secs() as u16;
    }

    pub fn change_volume(&mut self, volume: f32) {
        self.volume += volume;
        if self.volume < 0. {
            self.volume = 0.;
        } else if self.volume > 1. {
            self.volume = 1.;
        }
        self.sink.set_volume(self.volume)
    }
}
