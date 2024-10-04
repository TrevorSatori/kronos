use std::sync::atomic::Ordering;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
                self.on_key_event_song_list(key);
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
            },
            KeyCode::Down => {
                let _ = self.selected_artist_index.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
                self.selected_song_index.store(0, Ordering::SeqCst);
            },
            KeyCode::Home => {
                self.selected_artist_index.store(0, Ordering::SeqCst);
                self.selected_song_index.store(0, Ordering::SeqCst);
            },
            KeyCode::End => {
                self.selected_artist_index.store(len.saturating_sub(1), Ordering::SeqCst);
                self.selected_song_index.store(0, Ordering::SeqCst);
            },
            KeyCode::Delete => {
                let removed_artist = artists.remove(self.selected_artist_index.load(Ordering::SeqCst));
                let mut songs = self.songs.lock().unwrap();
                songs.remove(removed_artist.as_str());
            },
            _ => {},
        }
    }

    fn on_key_event_song_list(&self, key: KeyEvent) {
        let artists = self.artists.lock().unwrap();
        let artist = artists[self.selected_artist_index.load(Ordering::SeqCst)].as_str();

        let songs_all = self.songs.lock().unwrap();
        let songs = songs_all.get(artist);

        let Some(songs) = songs else {
            log::debug!("on_key_event_song_list No songs for {artist}");
            return;
        };

        let len = songs.len();

        match key.code {
            KeyCode::Up => {
                let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
            },
            KeyCode::Down => {
                let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
            },
            KeyCode::Home => {
                self.selected_song_index.store(0, Ordering::SeqCst);
            },
            KeyCode::End => {
                self.selected_song_index.store(len.saturating_sub(1), Ordering::SeqCst);
            },
            KeyCode::Enter | KeyCode::Char(_) => {
                let i = self.selected_song_index.load(Ordering::SeqCst);
                if i >= songs.len() {
                    log::error!("library on_key_event_song_list enter: selected_song_index > songs.len");
                    return;
                }
                let song = songs[self.selected_song_index.load(Ordering::SeqCst)].clone();
                self.on_select_fn.lock().unwrap()((song, key));
                let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
            },
            _ => {},
        }
    }

}