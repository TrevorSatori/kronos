use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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

    selected_artist_index: AtomicUsize,
    pub(super) selected_song_index: AtomicUsize,

    on_select_fn: Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>,
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
        self.songs.lock().unwrap().push(song);
        // self.selected_playlist_mut(move |pl| {
        //     pl.songs.push(song.clone());
        // });
    }

    pub fn add_cue(&self, cue_sheet: CueSheet) {
        let mut songs = Song::from_cue_sheet(cue_sheet);
        self.songs.lock().unwrap().append(&mut songs);
        // self.selected_playlist_mut(move |pl| {
        //     let mut songs = Song::from_cue_sheet(cue_sheet);
        //     pl.songs.append(&mut songs);
        // });
    }

    pub fn on_key_event_artist_list(&self, key: KeyEvent) {
        let len = self.songs.lock().unwrap().len();

        match key.code {
            KeyCode::Up => {
                let _ = self.selected_artist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
            },
            KeyCode::Down => {
                let _ = self.selected_artist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
            },
            KeyCode::Home => {
                self.selected_artist_index.store(0, Ordering::Relaxed);
            },
            KeyCode::End => {
                self.selected_artist_index.store(len.saturating_sub(1), Ordering::Relaxed);
            },
            // KeyCode::Delete => {
            //     let selected_playlist_index = self.selected_artist_index.load(Ordering::Relaxed);
            //     let mut playlists = self.playlists.lock().unwrap();
            //
            //     if playlists.len() > 0 {
            //         playlists.remove(selected_playlist_index);
            //         if selected_playlist_index > playlists.len().saturating_sub(1) {
            //             self.selected_artist_index.store(playlists.len().saturating_sub(1), Ordering::Relaxed);
            //         }
            //     }
            // }
            _ => {},
        }
    }

    pub fn on_key_event_song_list(&self, key: KeyEvent) {
        let len = self.songs.lock().unwrap().len();

        match key.code {
            KeyCode::Up if key.modifiers == KeyModifiers::NONE => {
                let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
            },
            KeyCode::Down if key.modifiers == KeyModifiers::NONE => {
                let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
            },
            KeyCode::Enter | KeyCode::Char(_) => {
                // let selected_song = self.selected_playlist(|pl| pl.songs[self.selected_song_index.load(Ordering::Relaxed)].clone());
                // if let Some(song) = selected_song {
                //     self.on_select_fn.lock().unwrap()((song, key));
                // }
                let selected_song_index = self.selected_song_index.load(Ordering::Relaxed);
                let song = self.songs.lock().unwrap()[selected_song_index].clone();
                self.on_select_fn.lock().unwrap()((song, key));
            },
            // KeyCode::Delete => {
            //     let selected_song = self.selected_song_index.load(Ordering::Relaxed);
            //     self.selected_playlist_mut(|pl| {
            //         if pl.songs.len() > 0 {
            //             pl.songs.remove(selected_song);
            //             if selected_song >= pl.songs.len() {
            //                 self.selected_song_index.store(selected_song.saturating_sub(1), Ordering::Relaxed);
            //             }
            //         }
            //     });
            // },
            _ => {},
        }
    }

}

impl Drop for Library<'_> {
    fn drop(&mut self) {
        log::trace!("Library.drop()");
    }
}
