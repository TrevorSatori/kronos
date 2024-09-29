use std::sync::atomic::Ordering;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    ui::KeyboardHandlerRef,
};

use super::library::{Library, LibraryScreenElement};

impl<'a> KeyboardHandlerRef<'a> for Library<'a> {

    fn on_key(&self, key: KeyEvent) -> bool {
        let focused_element_guard = self.focused_element.lock().unwrap();

        match key.code {
            // KeyCode::Tab => {
            //     *focused_element_guard = match *focused_element_guard {
            //         PlaylistScreenElement::PlaylistList => PlaylistScreenElement::SongList,
            //         PlaylistScreenElement::SongList => PlaylistScreenElement::PlaylistList,
            //     };
            // }
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

    fn on_key_event_song_list(&self, key: KeyEvent) {
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
