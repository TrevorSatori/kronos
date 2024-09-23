use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::Widget,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{WidgetRef},
};

use crate::{
    structs::{Song},
    config::Theme,
    cue::CueSheet,
    ui::song_to_string,
};

#[derive(Eq, PartialEq)]
enum LibraryScreenElement {
    ArtistList,
    SongList,
}

pub struct Library<'a> {
    songs: Mutex<Vec<Song>>,
    theme: Theme,
    focused_element: Mutex<LibraryScreenElement>,
    selected_artist_index: AtomicUsize,
    selected_song_index: AtomicUsize,
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

    pub fn on_key_event(&self, key: KeyEvent) {
        let mut focused_element_guard = self.focused_element.lock().unwrap();

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
            _ => {},
        }
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

impl<'a> Widget for Library<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl<'a> WidgetRef for Library<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_left, _, area_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
            .horizontal_margin(2)
            .areas(area);

        let songs = self.songs.lock().unwrap();
        if songs.len() < 1 {
            return;
        }

        // let selected_artist_index = self.selected_artist_index.load(Ordering::Relaxed);
        let selected_song_index = self.selected_song_index.load(Ordering::Relaxed);
        let focused_element = self.focused_element.lock().unwrap();

        for i in 0..songs.len() {
            let song = &songs[i];
            let area = Rect {
                y: area_left.y + i as u16,
                height: 1,
                ..area_left
            };

            let style = if i == selected_song_index {
                if *focused_element == LibraryScreenElement::ArtistList {
                    Style::default().fg(self.theme.highlight_foreground).bg(self.theme.highlight_background)
                } else {
                    Style::default().fg(self.theme.highlight_foreground).bg(Color::from_hsl(29.0, 54.0, 34.0))
                }
            } else {
                Style::default().fg(Color::White).bg(self.theme.background)
            };

            let line = ratatui::text::Line::from(song.title.as_str()).style(style);

            line.render_ref(area, buf);
        }
        //
        // if selected_song_index >= songs.len() {
        //     log::error!("selected_playlist_index >= playlists.len()");
        //     return;
        // }

    }
}
