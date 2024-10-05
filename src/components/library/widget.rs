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
        let [area_left, _, area_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
            .horizontal_margin(2)
            .areas(area);

        self.height.store(area.height as usize, Ordering::Relaxed);

        self.render_ref_artists(area_left, buf);
        self.render_ref_songs(area_right, buf);
    }
}

fn line_style(theme: &crate::config::Theme, index: usize, selected_index: usize, list_has_focus: bool) -> Style {
    if index == selected_index {
        if list_has_focus {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected)
        } else {
            Style::default().fg(theme.foreground_selected).bg(theme.background_selected_blur)
        }
    } else {
        Style::default().fg(theme.foreground_secondary).bg(theme.background)
    }
}

impl<'a> Library<'a> {
    fn render_ref_artists(&self, area: Rect, buf: &mut Buffer) {
        self.height.store(area.height as usize, Ordering::Relaxed);

        let focused_element = self.focused_element.lock().unwrap();
        let selected_artist_index = self.selected_artist_index.load(Ordering::Relaxed);
        let artists = self.artists.lock().unwrap();

        for i in 0..artists.len() {
            let artist = artists[i].as_str();
            let area = Rect {
                y: area.y + i as u16,
                height: 1,
                ..area
            };

            let style = line_style(&self.theme, i, selected_artist_index, *focused_element == LibraryScreenElement::ArtistList);
            let line = ratatui::text::Line::from(artist).style(style);

            line.render_ref(area, buf);
        }
    }

    fn render_ref_songs(&self, area: Rect, buf: &mut Buffer) {
        let selected_artist_index = self.selected_artist_index.load(Ordering::Relaxed);
        let artists = self.artists.lock().unwrap();

        if selected_artist_index >= artists.len() {
            log::error!("selected_artist_index >= artists.len()");
            return;
        }

        let songs = self.selected_artist_songs.lock().unwrap();

        if songs.len() < 1 {
            return;
        }

        let focused_element = self.focused_element.lock().unwrap();

        let selected_song_index = self.selected_song_index.load(Ordering::Relaxed);
        let offset = self.offset.load(Ordering::Relaxed);

        for i in 0..songs.len().min(area.height as usize) {
            let song_index = i + offset;

            if song_index >= songs.len() {
                log::error!("song index {song_index} > songs.len() {} offset={offset}", songs.len());
                break;
            }

            let song = &songs[song_index];
            let area = Rect {
                y: area.y + i as u16,
                height: 1,
                ..area
            };

            let style = line_style(&self.theme, song_index, selected_song_index, *focused_element == LibraryScreenElement::SongList);
            let line = ratatui::text::Line::from(
                format!("{} - {} - {}",
                        song.album.clone().unwrap_or("(no album)".to_string()),
                        song.track.unwrap_or(0),
                        song.title.clone()
                ),
            ).style(style);

            line.render_ref(area, buf);
        }
    }
}
