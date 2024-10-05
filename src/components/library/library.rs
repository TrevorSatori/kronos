use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    rc::Rc,
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
    ui::KeyboardHandlerRef,
};
use super::song_list::SongList;

#[derive(Eq, PartialEq)]
pub(super) enum LibraryScreenElement {
    ArtistList,
    SongList,
}

pub struct Library<'a> {
    pub(super) theme: Theme,

    pub(super) artists: Arc<Mutex<Vec<String>>>,
    pub(super) songs: Mutex<HashMap<String, Vec<Song>>>,
    pub(super) song_list: Mutex<SongList<'a>>,

    pub(super) focused_element: Mutex<LibraryScreenElement>,

    pub(super) selected_artist_index: AtomicUsize,
    pub(super) selected_song_index: AtomicUsize,

    pub(super) on_select_fn: Rc<Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>>,

    pub(super) offset: AtomicUsize,
    pub(super) height: AtomicUsize,
}

impl<'a> Library<'a> {
    pub fn new(theme: Theme, songs: Vec<Song>) -> Self {
        let on_select_fn: Rc<Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>> = Rc::new(Mutex::new(Box::new(|_| {}) as _));

        let songs_el = SongList::new(theme);
        songs_el.on_select({
            let on_select_fn = on_select_fn.clone();
            move |(song, key)| {
                log::debug!("song selected {:?}", song);
                let mut on_select_fn = on_select_fn.lock().unwrap();

                on_select_fn((song, key));
            }
        });

        let lib = Self {
            theme,
            focused_element: Mutex::new(LibraryScreenElement::ArtistList),

            on_select_fn,

            artists: Arc::new(Mutex::new(vec![])),
            songs: Mutex::new(HashMap::new()),
            song_list: Mutex::new(songs_el),

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

        let Some(selected_artist) = artists.get(self.selected_song_index.load(AtomicOrdering::SeqCst)) else {
            return;
        };

        if artist != *selected_artist {
            return;
        }

        let Some(artist_songs) = songs.get(artist.as_str()) else {
            log::error!("No song list for {artist}! This is an error: we should always have a Vec<Song>, even if empty.");
            return;
        };

        // Cloning the song list once per key press is probably more performant than dealing with Rc's, WeakRef's and whatnot.
        let artist_songs = artist_songs.iter().map(|s| s.clone()).collect();
        self.song_list.lock().unwrap().set_songs(artist_songs);
    }

    pub fn add_cue(&self, cue_sheet: CueSheet) {
        let songs = Song::from_cue_sheet(cue_sheet);
        self.add_songs(songs);
    }

    pub fn add_directory(&self, path: &PathBuf) {
        let songs = Song::from_dir(path);
        self.add_songs(songs);
    }

}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
