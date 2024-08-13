use std::{fs::File, io::BufReader, sync::Arc, thread, time::Duration};

use crate::helpers::gen_funcs::Song;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct MusicHandle {
    music_output: (OutputStream, OutputStreamHandle),
    sink: Arc<Sink>,
    currently_playing: Option<Song>,
    volume: f32,
}

impl Default for MusicHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicHandle {
    pub fn new() -> Self {
        let music_output = OutputStream::try_default().unwrap();
        let sink = Arc::new(Sink::try_new(&music_output.1).unwrap());
        Self {
            music_output,
            sink,
            currently_playing: None,
            volume: 1.,
        }
    }

    pub fn sink(&self) -> Arc<Sink> {
        self.sink.clone()
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

    pub fn sink_empty(&self) -> bool {
        self.sink.empty()
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

    pub fn change_volume(&mut self, volume: f32) {
        self.volume += volume;
        if self.volume < 0. {
            self.volume = 0.;
        } else if self.volume > 1. {
            self.volume = 1.;
        }
        self.sink.set_volume(self.volume)
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
