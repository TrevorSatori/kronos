use std::sync::atomic::Ordering;

use ratatui::{
    prelude::Widget,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{WidgetRef},
};

use super::{Library, LibraryScreenElement};

impl<'a> Widget for Library<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl<'a> WidgetRef for Library<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_left, _, _area_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
            .horizontal_margin(2)
            .areas(area);

        let focused_element = self.focused_element.lock().unwrap();
        let selected_artist_index = self.selected_artist_index.load(Ordering::Relaxed);


        let artists = self.artists.lock().unwrap();

        for i in 0..artists.len() {
            let artist = artists[i].as_str();
            let area = Rect {
                y: area_left.y + i as u16,
                height: 1,
                ..area_left
            };

            let style = if i == selected_artist_index {
                if *focused_element == LibraryScreenElement::ArtistList {
                    Style::default().fg(self.theme.highlight_foreground).bg(self.theme.highlight_background)
                } else {
                    Style::default().fg(self.theme.highlight_foreground).bg(Color::from_hsl(29.0, 54.0, 34.0))
                }
            } else {
                Style::default().fg(Color::White).bg(self.theme.background)
            };

            let line = ratatui::text::Line::from(artist).style(style);

            line.render_ref(area, buf);
        }


        if selected_artist_index >= artists.len() {
            log::error!("selected_artist_index >= artists.len()");
            return;
        }

        let artist = artists[selected_artist_index].as_str();
        let songs_all = self.songs.lock().unwrap();
        let songs = songs_all.get(artist);

        let Some(songs) = songs else {
            return;
        };

        if songs.len() < 1 {
            return;
        }

        let selected_song_index = self.selected_song_index.load(Ordering::Relaxed);
        for i in 0..songs.len() {
            let song = &songs[i];
            let area = Rect {
                y: _area_right.y + i as u16,
                height: 1,
                .._area_right
            };

            let style = if i == selected_song_index {
                if *focused_element == LibraryScreenElement::SongList {
                    Style::default().fg(self.theme.highlight_foreground).bg(self.theme.highlight_background)
                } else {
                    Style::default().fg(self.theme.highlight_foreground).bg(Color::from_hsl(29.0, 54.0, 34.0))
                }
            } else {
                Style::default().fg(Color::White).bg(self.theme.background)
            };

            let line = ratatui::text::Line::from(format!("{} - {}", song.album.clone().unwrap_or("(no album)".to_string()), song.title.clone())).style(style);

            line.render_ref(area, buf);
        }
        //
        // if selected_song_index >= songs.len() {
        //     log::error!("selected_playlist_index >= playlists.len()");
        //     return;
        // }

    }
}
