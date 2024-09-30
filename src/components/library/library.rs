use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize},
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

#[derive(Eq, PartialEq)]
pub(super) enum LibraryScreenElement {
    ArtistList,
    SongList,
}

pub struct Library<'a> {
    pub(super) theme: Theme,

    pub(super) artists: Arc<Mutex<Vec<String>>>, // TODO: songs: Vec<Song>, other stuff: weak ref, etc
    pub(super) songs: Mutex<HashMap<String, Vec<Song>>>,

    pub(super) focused_element: Mutex<LibraryScreenElement>,

    pub(super) selected_artist_index: AtomicUsize,
    pub(super) selected_song_index: AtomicUsize,

    pub(super) on_select_fn: Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>,
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme, songs: Vec<Song>) -> Self {
        let lib = Self {
            theme,
            focused_element: Mutex::new(LibraryScreenElement::SongList),

            on_select_fn: Mutex::new(Box::new(|_| {}) as _),

            artists: Arc::new(Mutex::new(vec![])),
            songs: Mutex::new(HashMap::new()),
            selected_artist_index: AtomicUsize::new(0),
            selected_song_index: AtomicUsize::new(0),
        };

        lib.add_songs(songs);

        lib
    }

    pub fn on_select(&self, cb: impl FnMut((Song, KeyEvent)) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn songs(&self) -> Vec<Song> {
        let mut songs = vec![];

        for (_artist, artist_songs) in &*self.songs.lock().unwrap() {
            for song in artist_songs {
                songs.push(song.clone());
            }
        }

        songs
    }

    pub fn add_songs(&self, songs: Vec<Song>) {
        for song in songs {
            self.add_song(song);
        }
    }

    pub fn add_song(&self, song: Song) {
        let mut songs = self.songs.lock().unwrap();

        let Some(artist) = song.artist.clone() else {
            log::error!("Library.add_song() -> no artist! {:?}", song);
            return;
        };

        if let Some(mut x) = songs.get_mut(&artist) {
            x.push(song);
        } else {
            songs.insert(artist.clone(), vec![song]);
        }

        let mut artists = self.artists.lock().unwrap();

        if !artists.contains(&artist) {
            (*artists).push(artist.clone());
        }
    }

    pub fn add_cue(&self, cue_sheet: CueSheet) {
        let songs = Song::from_cue_sheet(cue_sheet);
        self.add_songs(songs);
    }

    pub fn add_directory(&self, path: PathBuf) {

    }

}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
