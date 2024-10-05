use std::sync::atomic::Ordering;
use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    ui::KeyboardHandlerRef,
};

use super::library::{Library, LibraryScreenElement};

impl<'a> KeyboardHandlerRef<'a> for Library<'a> {

    fn on_key(&self, key: KeyEvent) -> bool {
        let mut focused_element_guard = self.focused_element.lock().unwrap();

        match key.code {
            KeyCode::Tab => {
                *focused_element_guard = match &*focused_element_guard {
                    LibraryScreenElement::ArtistList => LibraryScreenElement::SongList,
                    LibraryScreenElement::SongList => LibraryScreenElement::ArtistList,
                };
            }
            _ if *focused_element_guard == LibraryScreenElement::ArtistList  => {
                self.on_key_event_artist_list(key);
            },
            _ if *focused_element_guard == LibraryScreenElement::SongList  => {
                self.song_list.lock().unwrap().on_key(key);
            },
            _ => {
                return false;
            },
        }

        true
    }
}


impl<'a> Library<'a> {

    fn on_key_event_artist_list(&self, key: KeyEvent) {
        let mut artists = self.artists.lock().unwrap();
        let len = artists.len();

        match key.code {
            KeyCode::Up => {
                let _ = self.selected_artist_index.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |a| { Some(a.saturating_sub(1)) });
                self.selected_song_index.store(0, Ordering::SeqCst);
                self.offset.store(0, Ordering::SeqCst);
            },
            KeyCode::Down => {
                let _ = self.selected_artist_index.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
                self.selected_song_index.store(0, Ordering::SeqCst);
                self.offset.store(0, Ordering::SeqCst);
            },
            KeyCode::Home => {
                self.selected_artist_index.store(0, Ordering::SeqCst);
                self.selected_song_index.store(0, Ordering::SeqCst);
                self.offset.store(0, Ordering::SeqCst);
            },
            KeyCode::End => {
                self.selected_artist_index.store(len.saturating_sub(1), Ordering::SeqCst);
                self.selected_song_index.store(0, Ordering::SeqCst);
                self.offset.store(0, Ordering::SeqCst);
            },
            KeyCode::Delete => {
                let removed_artist = artists.remove(self.selected_artist_index.load(Ordering::SeqCst));
                let mut songs = self.songs.lock().unwrap();
                songs.remove(removed_artist.as_str());
                self.offset.store(0, Ordering::SeqCst);
                let _ = self.selected_artist_index.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |a| { Some(a.saturating_sub(1).min(len.saturating_sub(1))) });
            },
            _ => {},
        }


        let artist = artists[self.selected_artist_index.load(Ordering::SeqCst)].as_str();
        let songs = self.songs.lock().unwrap();
        let artist_songs = songs.get(artist).unwrap();
        let artist_songs = artist_songs.iter().map(|s| s.clone()).collect();
        self.song_list.lock().unwrap().set_songs(artist_songs);
    }

}
