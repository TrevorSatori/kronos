use std::sync::atomic::Ordering;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    ui::KeyboardHandlerRef,
};

use super::component::SongList;

impl<'a> KeyboardHandlerRef<'a> for SongList<'a> {

    fn on_key(&self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End => {
                self.on_song_list_directional_key(key);
            },
            KeyCode::Enter | KeyCode::Char(_) => {
                let songs = &self.songs.lock().unwrap();
                let len = songs.len();

                let i = self.selected_song_index.load(Ordering::SeqCst);
                if i >= songs.len() {
                    log::error!("library on_key_event_song_list enter: selected_song_index > song_list.len");
                    return true;
                }
                let song = songs[self.selected_song_index.load(Ordering::SeqCst)].clone();
                self.on_select_fn.lock().unwrap()((song, key));
                let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
            },
            _ => {},
        }

        true
    }
}


impl<'a> SongList<'a> {

    fn on_song_list_directional_key(&self, key: KeyEvent) {
        let songs = &self.songs.lock().unwrap();
        let length = songs.len() as i32;

        let height = self.height.load(Ordering::Relaxed) as i32;
        let padding = 5;

        let mut offset = self.offset.load(Ordering::SeqCst) as i32;
        let mut i = self.selected_song_index.load(Ordering::SeqCst) as i32;

        match key.code {
            KeyCode::Up if key.modifiers == KeyModifiers::NONE  => {
                i -= 1;
                if i < offset + padding {
                    offset = if i > padding {
                        i - padding
                    } else {
                        0
                    };
                }

            },
            KeyCode::Up if key.modifiers == KeyModifiers::ALT => {
                let Some(song) = (*songs).get(i as usize) else {
                    log::error!("no selected song");
                    return;
                };

                let Some(ref selected_album) = song.album else {
                    log::warn!("no selected song album");
                    return;
                };

                let next_song = songs.iter().take(i as usize).rposition(|s| s.album.as_ref().is_some_and(|a| a != selected_album));
                let next_song_album = next_song.and_then(|ns| songs.get(ns)).and_then(|ref s| s.album.as_ref());

                if let Some(next_song_album) = next_song_album {
                    let next_song = songs.iter().position(|ref s| s.album.as_ref().is_some_and(|a| a.as_str() == next_song_album));
                    if let Some(next_song) = next_song {
                        i = next_song as i32;
                    }
                }
            },
            KeyCode::Home => {
                i = 0;
                offset = 0;
            },
            KeyCode::Down if key.modifiers == KeyModifiers::NONE => {
                let padding = height.saturating_sub(padding).saturating_sub(1);
                i += 1;
                if i > offset + padding {
                    offset = if i > padding {
                        i - padding
                    } else {
                        0
                    };
                }

            },
            KeyCode::Down if key.modifiers == KeyModifiers::ALT => {
                let Some(song) = (*songs).get(i as usize) else {
                    log::error!("no selected song");
                    return;
                };

                let Some(ref selected_album) = song.album else {
                    log::warn!("no selected song album");
                    return;
                };

                let next_song = songs.iter().skip(i as usize).position(|s| s.album.as_ref().is_some_and(|a| a != selected_album));
                let next_song = next_song.map(|ns| ns.saturating_add(i as usize));

                if let Some(next_song) = next_song {
                    i = next_song as i32;
                }
            },
            KeyCode::End => {
                i = length - 1;
                offset = i - height + padding;
            },
            _ => {},
        }

        offset = offset.min(length - height).max(0);
        i = i.min(length - 1).max(0);

        // log::debug!("asd {offset} {i}");
        self.offset.store(offset as usize, Ordering::SeqCst);
        self.selected_song_index.store(i as usize, Ordering::SeqCst);
    }

}
