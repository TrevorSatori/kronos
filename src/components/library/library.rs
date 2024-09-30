use std::{
    sync::{
        atomic::{AtomicUsize},
        Mutex,
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
    pub(super) songs: Mutex<Vec<Song>>,

    pub(super) focused_element: Mutex<LibraryScreenElement>,

    pub(super) selected_artist_index: AtomicUsize,
    pub(super) selected_song_index: AtomicUsize,

    pub(super) on_select_fn: Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>,
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme, songs: Vec<Song>) -> Self {
        Self {
            songs: Mutex::new(songs),
            selected_artist_index: AtomicUsize::new(0),
            selected_song_index: AtomicUsize::new(0),
            theme,
            focused_element: Mutex::new(LibraryScreenElement::SongList),
            on_select_fn: Mutex::new(Box::new(|_| {}) as _),
        }
    }

    pub fn songs(&self) -> Vec<Song> {
        self.songs.lock().unwrap().clone()
    }

    pub fn on_select(&self, cb: impl FnMut((Song, KeyEvent)) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    // pub fn artists(&self) -> Vec<Song> {
    //     let artists = self.artists.lock().unwrap();
    //     artists.clone()
    // }

    // pub fn selected_artist<T>(&self, f: impl FnOnce(&Playlist) -> T) -> Option<T> {
    //     let selected_playlist_index = self.selected_artist_index.load(Ordering::Relaxed);
    //     let mut playlists = self.playlists.lock().unwrap();
    //
    //     if let Some(selected_playlist) = playlists.get_mut(selected_playlist_index) {
    //         Some(f(selected_playlist))
    //     } else {
    //         None
    //     }
    // }

    // pub fn selected_playlist_mut(&self, f: impl FnOnce(&mut Playlist)) {
    //     let selected_playlist_index = self.selected_artist_index.load(Ordering::Relaxed);
    //     let mut playlists = self.playlists.lock().unwrap();
    //
    //     if let Some(selected_playlist) = playlists.get_mut(selected_playlist_index) {
    //         f(selected_playlist);
    //     }
    // }

    pub fn add_song(&self, song: Song) {
        let mut songs = self.songs.lock().unwrap();

        if songs.iter().find(|p| p.path == song.path).is_some() {
            log::debug!("Library.add_song(): already in library! {}", song.path.display());
            return;
        }

        songs.push(song);
    }

    pub fn add_cue(&self, cue_sheet: CueSheet) {
        let mut songs = Song::from_cue_sheet(cue_sheet);
        self.songs.lock().unwrap().append(&mut songs);
    }

    pub fn add_directory(&self, path: PathBuf) {

    }

}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
