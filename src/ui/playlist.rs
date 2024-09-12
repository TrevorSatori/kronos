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

pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
}

pub struct Playlists {
    pub playlists: Mutex<Vec<Playlist>>,
    pub selected_playlist_index: AtomicUsize,
    theme: Theme,
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
            theme,
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

    pub fn on_key_event(&self, key: KeyEvent) {
        let len = self.playlists.lock().unwrap().len();

        match key.code {
            KeyCode::Up => {
                let _ = self.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
            },
            KeyCode::Down => {
                let _ = self.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
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

        for i in 0..playlists.len() {
            let playlist = &playlists[i];
            let area = Rect {
                y: area_left.y + i as u16,
                height: 1,
                ..area_left
            };

            let style = if i == selected_playlist {
                Style::default().fg(self.theme.highlight_foreground).bg(self.theme.highlight_background)
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
