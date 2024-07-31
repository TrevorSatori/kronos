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
    song_length: u32,
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
            currently_playing: "CURRENT SONG".to_string(),
            volume: 1.,
        }
    }

    pub fn currently_playing(&self) -> String {
        self.currently_playing.clone()
    }

    pub fn song_length(&self) -> u32 {
        self.song_length
    }

    pub fn time_played(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn sink_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn set_currently_playing(&mut self, path: &PathBuf) {
        self.currently_playing = gen_funcs::audio_display(path);
    }

    pub fn play(&mut self, path: PathBuf) {
        self.sink.stop();

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

        let _t1 = thread::spawn(move || {
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();

            sclone.append(source);

            sclone.sleep_until_end();
            eprintln!("Song finished playing");
            // TODO: notify something so we can auto_play here rather than randomly probing
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
        self.song_length = duration.as_secs() as u32;
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
