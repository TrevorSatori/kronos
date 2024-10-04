use std::{
    cmp::Ordering,
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

    pub(super) offset: AtomicUsize,
    pub(super) height: AtomicUsize,
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme, songs: Vec<Song>) -> Self {
        let lib = Self {
            theme,
            focused_element: Mutex::new(LibraryScreenElement::ArtistList),

            on_select_fn: Mutex::new(Box::new(|_| {}) as _),

            artists: Arc::new(Mutex::new(vec![])),
            songs: Mutex::new(HashMap::new()),
            selected_artist_index: AtomicUsize::new(0),
            selected_song_index: AtomicUsize::new(0),

            offset: AtomicUsize::new(0),
            height: AtomicUsize::new(0),
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
        // log::debug!(target: "::library.add_song", "{:?}", song);

        let mut songs = self.songs.lock().unwrap();

        let Some(artist) = song.artist.clone() else {
            log::error!("Library.add_song() -> no artist! {:?}", song);
            return;
        };

        if let Some(mut x) = songs.get_mut(&artist) {
            if !x.iter().any(|s| s.path == song.path && s.title == song.title) {
                x.push(song);
                x.sort_by(|a, b| {
                    match (&a.album, &b.album) {
                        (Some(album_a), Some(album_b)) if album_a == album_b => {
                            match (&a.track, &b.track) {
                                (Some(a), Some(b)) => a.cmp(b),
                                (Some(_), None) => Ordering::Greater,
                                (None, Some(_)) => Ordering::Less,
                                _ => a.title.cmp(&b.title),
                            }
                        },
                        (Some(album_a), Some(album_b)) if album_a != album_b => {
                            album_a.cmp(album_b)
                        },
                        (Some(_), None) => Ordering::Greater,
                        (None, Some(_)) => Ordering::Less,
                        _ => a.title.cmp(&b.title)
                    }
                })
            }
        } else {
            songs.insert(artist.clone(), vec![song]);
        }

        let mut artists = self.artists.lock().unwrap();

        if !artists.contains(&artist) {
            (*artists).push(artist.clone());
        }

        artists.sort_unstable();
    }

    pub fn add_cue(&self, cue_sheet: CueSheet) {
        let songs = Song::from_cue_sheet(cue_sheet);
        self.add_songs(songs);
    }

    pub fn add_directory(&self, path: &PathBuf) {
        let songs = Song::from_dir(path);
        self.add_songs(songs);
    }

    pub fn selected_artist_index(&self) -> usize {
        self.selected_artist_index.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn selected_artist(&self) -> String {
        let artists = self.artists.lock().unwrap();
        artists[self.selected_artist_index()].clone()
    }

    pub fn songs_by_artist(&self) -> Option<Vec<Song>>{// TODO: WeakRef, Arc / Rc
        let artist = self.selected_artist();
        let songs = self.songs.lock().unwrap();
        let artist_songs = (*songs).get(artist.as_str());

        artist_songs.map(|s| s.clone())
    }
}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
