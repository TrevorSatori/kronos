use std::{fs::File, io::BufReader, sync::Arc, thread, time::Duration};

use crate::helpers::gen_funcs::Song;
use rodio::{Decoder, OutputStreamHandle, Sink};

pub struct MusicHandle {
    sink: Arc<Sink>,
    currently_playing: Option<Song>,
}

impl MusicHandle {
    pub fn new(sink: Arc<Sink>) -> Self {
        Self {
            sink,
            currently_playing: None,
        }
    }

    pub fn currently_playing(&self) -> Option<Song> {
        self.currently_playing.clone()
    }

    pub fn song_length(&self) -> Duration {
        match &self.currently_playing {
            Some(song) => song.length,
            _ => Duration::from_secs(0),
        }
    }

    pub fn time_played(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn play(&mut self, song: Song) {
        self.sink.stop();

        let path = song.path.clone();
        self.currently_playing = Some(song);

        let sink = self.sink.clone();

        let _t1 = thread::spawn(move || {
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();

            sink.append(source);
            sink.sleep_until_end();
            // TODO: let (tx, rx) = channel(); (see sink.sleep_until_end implementation)
        });
    }

    pub fn skip(&self) {
        self.sink.stop();
    }

    pub fn seek_forward(&mut self) {
        let target = self
            .sink
            .get_pos()
            .saturating_add(Duration::from_secs(5))
            .min(self.song_length());
        self.sink.try_seek(target).unwrap_or_else(|e| {
            eprintln!("could not seek {:?}", e);
        });
    }

    pub fn seek_backward(&mut self) {
        let target = self
            .sink
            .get_pos()
            .saturating_sub(Duration::from_secs(5))
            .max(Duration::from_secs(0));
        self.sink.try_seek(target).unwrap_or_else(|e| {
            eprintln!("could not seek {:?}", e);
        });
    }
}

pub trait ExtendedSink {
    fn toggle(&self);
    fn change_volume(&self, amount: f32);
}

impl ExtendedSink for Sink {
    fn toggle(&self) {
        if self.is_paused() {
            self.play()
        } else {
            self.pause()
        }
    }

    fn change_volume(&self, amount: f32) {
        let mut volume = self.volume() + amount;
        if volume < 0. {
            volume = 0.;
        } else if volume > 1. {
            volume = 1.;
        }
        self.set_volume(volume)
    }
}