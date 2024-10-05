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
        self.song_list.lock().unwrap().render_ref(area_right, buf);
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

}
