use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::Widget,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{WidgetRef},
};

use crate::{
    ui,
    structs::Song,
    config::Theme,
};
use crate::cue::CueSheet;

pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
}

#[derive(Eq, PartialEq)]
enum PlaylistScreenElement {
    PlaylistList,
    SongList,
}

pub struct Playlists {
    pub playlists: Mutex<Vec<Playlist>>,
    theme: Theme,
    focused_element: Mutex<PlaylistScreenElement>,
    selected_playlist_index: AtomicUsize,
    selected_song_index: AtomicUsize,
}

impl Playlists {
    pub fn new(theme: Theme) -> Self {
        Self {
            playlists: Mutex::new(vec![
                Playlist {
                    name: "Cream".to_string(),
                    songs: vec![]
                },
                Playlist {
                    name: "ASD".to_string(),
                    songs: vec![]
                },
                Playlist {
                    name: "xxx".to_string(),
                    songs: vec![]
                },
                Playlist {
                    name: "aaaaaaaaaaaa".to_string(),
                    songs: vec![]
                },
            ]),
            selected_playlist_index: AtomicUsize::new(0),
            selected_song_index: AtomicUsize::new(0),
            theme,
            focused_element: Mutex::new(PlaylistScreenElement::PlaylistList),
        }
    }

    // pub fn selected_playlist(&self) -> Option<&Playlist> {
    //     // &self.playlists[self.selected_playlist_index]
    //     self.selected_playlist
    // }

    pub fn add_song(&self, song: Song) {
        let selected_playlist_index = self.selected_playlist_index.load(Ordering::Relaxed);
        self.playlists.lock().unwrap()[selected_playlist_index].songs.push(song);
    }

    pub fn add_cue(&self, cue_sheet: CueSheet) {
        let selected_playlist_index = self.selected_playlist_index.load(Ordering::Relaxed);
        let mut playlists = self.playlists.lock().unwrap();

        if let Some(selected_playlist) = playlists.get_mut(selected_playlist_index) {
            let mut songs = Song::from_cue_sheet(cue_sheet);
            selected_playlist.songs.append(&mut songs);
        }
    }

    pub fn on_key_event(&self, key: &KeyEvent) {
        let len = self.playlists.lock().unwrap().len();
        let mut focused_element_guard = self.focused_element.lock().unwrap();

        match key.code {
            KeyCode::Tab => {
                log::debug!("TAB");
                *focused_element_guard = match *focused_element_guard {
                    PlaylistScreenElement::PlaylistList => PlaylistScreenElement::SongList,
                    PlaylistScreenElement::SongList => PlaylistScreenElement::PlaylistList,
                };
            }
            KeyCode::Up => {
                match *focused_element_guard {
                    PlaylistScreenElement::PlaylistList => {
                        let _ = self.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
                    },
                    PlaylistScreenElement::SongList => {
                        let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
                    },
                };

            },
            KeyCode::Down => {
                match *focused_element_guard {
                    PlaylistScreenElement::PlaylistList => {
                        let _ = self.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
                    },
                    PlaylistScreenElement::SongList => {
                        let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
                    },
                };
            },
            _ => {},
        }
    }

}

impl Widget for Playlists {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl WidgetRef for Playlists {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_left, _, area_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
            .horizontal_margin(2)
            .areas(area);

        let playlists = self.playlists.lock().unwrap();
        let selected_playlist = self.selected_playlist_index.load(Ordering::Relaxed);
        let focused_element = self.focused_element.lock().unwrap();

        for i in 0..playlists.len() {
            let playlist = &playlists[i];
            let area = Rect {
                y: area_left.y + i as u16,
                height: 1,
                ..area_left
            };

            let style = if i == selected_playlist {
                if *focused_element == PlaylistScreenElement::PlaylistList {
                    Style::default().fg(self.theme.highlight_foreground).bg(self.theme.highlight_background)
                } else {
                    Style::default().fg(self.theme.highlight_foreground).bg(Color::from_hsl(29.0, 54.0, 34.0))
                }
            } else {
                Style::default().fg(Color::White).bg(self.theme.background)
            };

            let line = ratatui::text::Line::from(playlist.name.as_str()).style(style);

            line.render_ref(area, buf);
        }

        let playlist = &playlists[selected_playlist];
        let songs = playlist.songs.iter().map(ui::song_to_string).collect();
        let list = ratatui::widgets::List::from(songs);

        list.render(area_right, buf);
    }
}
