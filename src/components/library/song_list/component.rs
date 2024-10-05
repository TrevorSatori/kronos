use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
        Mutex,
        Arc,
    },
    path::PathBuf,
};

use crossterm::event::KeyEvent;

use crate::{
    structs::{Song},
    config::Theme,
    cue::CueSheet,
};

pub struct SongList<'a> {
    pub(super) theme: Theme,

    pub(super) songs: Mutex<Vec<Song>>,
    pub(super) selected_song_index: AtomicUsize,

    pub(super) on_select_fn: Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>,

    pub(super) offset: AtomicUsize,
    pub(super) height: AtomicUsize,
}

impl<'a> SongList<'a> {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,

            on_select_fn: Mutex::new(Box::new(|_| {}) as _),

            songs: Mutex::new(Vec::new()),
            selected_song_index: AtomicUsize::new(0),

            offset: AtomicUsize::new(0),
            height: AtomicUsize::new(0),
        }
    }

    pub fn on_select(&self, cb: impl FnMut((Song, KeyEvent)) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn set_songs(&self, songs: Vec<Song>) {
        *self.songs.lock().unwrap() = songs;
    }
}

impl Drop for SongList<'_> {
    fn drop(&mut self) {
        log::trace!("Songs.drop()");
    }
}
