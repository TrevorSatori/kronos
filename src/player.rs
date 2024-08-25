use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rodio::{Decoder, OutputStreamHandle, Sink};

use crate::{
    helpers::{
        gen_funcs::{Song},
        queue::Queue,
    },
};

pub struct Player {
    sink: Arc<Sink>,
    currently_playing: Arc<Mutex<Option<Song>>>,
    queue_items: Arc<Queue>,
}

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
        }
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue_items.clone()
    }

    pub fn get_pos(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn currently_playing(&self) -> Arc<Mutex<Option<Song>>> {
        self.currently_playing.clone()
    }

    pub fn spawn_player_thread(&self) {
        let sink = self.sink.clone();
        let currently_playing = self.currently_playing.clone();
        let queue_items = self.queue_items.clone();

        thread::spawn(move || {
            loop {
                let song = queue_items.pop();
                let path = song.path.clone();

                match currently_playing.lock() {
                    Ok(mut s) => {
                        eprintln!("s {:?}", s);
                        *s = Some(song);
                    }
                    Err(err) => {
                        eprintln!("err {:?}", err);
                    }
                }

                let file = BufReader::new(File::open(path).unwrap());
                let source = Decoder::new(file).unwrap();

                sink.append(source);
                sink.sleep_until_end();

            }
        });
    }

    pub fn play_now(&self, song: Song) {
        self.queue_items.add_front(song);
        self.sink.stop();
    }

    pub fn toggle(&self) {
        if self.sink.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    pub fn stop(&self) {
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
                .min(song.length);
            self.sink.try_seek(target).unwrap_or_else(|e| {
                eprintln!("could not seek {:?}", e);
            });
        }
    }

    pub fn seek_backward(&self) {
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
