use crossterm::event::KeyEvent;

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
