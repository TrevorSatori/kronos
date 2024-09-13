use std::{
    time::{SystemTime, UNIX_EPOCH},
    sync::{
        atomic::{AtomicUsize, AtomicBool, Ordering},
        Mutex,
    },
};

use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::Widget,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{WidgetRef},
};

use crate::{
    ui,
    structs::{Song, Playlist},
    config::Theme,
    cue::CueSheet,
};
use crate::file_browser::FileBrowserSelection;

#[derive(Eq, PartialEq)]
enum PlaylistScreenElement {
    PlaylistList,
    SongList,
}

pub struct Playlists<'a> {
    playlists: Mutex<Vec<Playlist>>,
    theme: Theme,
    focused_element: Mutex<PlaylistScreenElement>,
    selected_playlist_index: AtomicUsize,
    selected_song_index: AtomicUsize,
    renaming: AtomicBool,
    on_select_fn: Mutex<Box<dyn FnMut((Song, KeyEvent)) + 'a>>,
}

impl<'a> Playlists<'a> {
    pub fn new(theme: Theme, playlists: Vec<Playlist>) -> Self {
        Self {
            // playlists: Mutex::new(vec![
            //     Playlist::new("My first Jolteon playlist".to_string()),
            //     Playlist::new("Ctrl+N to create new ones".to_string()),
            //     Playlist::new("Alt+N to rename".to_string()),
            // ]),
            playlists: Mutex::new(playlists),
            selected_playlist_index: AtomicUsize::new(0),
            selected_song_index: AtomicUsize::new(0),
            theme,
            focused_element: Mutex::new(PlaylistScreenElement::PlaylistList),
            renaming: AtomicBool::new(false),
            on_select_fn: Mutex::new(Box::new(|_| {}) as _),
        }
    }

    pub fn on_select(&self, cb: impl FnMut((Song, KeyEvent)) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn playlists(&self) -> Vec<Playlist> {
        let playlists = self.playlists.lock().unwrap();
        playlists.clone()
    }

    pub fn create_playlist(&self) {
        let playlist = Playlist {
            name: format!("New playlist created at {}", Local::now().format("%A %-l:%M:%S%P").to_string()),
            songs: vec![],
        };
        self.playlists.lock().unwrap().push(playlist);
    }

    pub fn selected_playlist<T>(&self, f: impl FnOnce(&Playlist) -> T) -> Option<T> {
        let selected_playlist_index = self.selected_playlist_index.load(Ordering::Relaxed);
        let mut playlists = self.playlists.lock().unwrap();

        if let Some(selected_playlist) = playlists.get_mut(selected_playlist_index) {
            Some(f(selected_playlist))
        } else {
            None
        }
    }

    pub fn selected_playlist_mut(&self, f: impl FnOnce(&mut Playlist)) {
        let selected_playlist_index = self.selected_playlist_index.load(Ordering::Relaxed);
        let mut playlists = self.playlists.lock().unwrap();

        if let Some(selected_playlist) = playlists.get_mut(selected_playlist_index) {
            f(selected_playlist);
        }
    }

    pub fn add_song(&self, song: Song) {
        self.selected_playlist_mut(move |pl| {
            pl.songs.push(song.clone());
        });
    }

    pub fn add_cue(&self, cue_sheet: CueSheet) {
        self.selected_playlist_mut(move |pl| {
            let mut songs = Song::from_cue_sheet(cue_sheet);
            pl.songs.append(&mut songs);
        });
    }

    pub fn on_key_event(&self, key: KeyEvent) {
        let mut focused_element_guard = self.focused_element.lock().unwrap();

        match key.code {
            KeyCode::Tab => {
                *focused_element_guard = match *focused_element_guard {
                    PlaylistScreenElement::PlaylistList => PlaylistScreenElement::SongList,
                    PlaylistScreenElement::SongList => PlaylistScreenElement::PlaylistList,
                };
            }
            _ if *focused_element_guard == PlaylistScreenElement::PlaylistList  => {
                self.on_key_event_playlist_list(key);
            },
            _ if *focused_element_guard == PlaylistScreenElement::SongList  => {
                self.on_key_event_song_list(key);
            },
            _ => {},
        }
    }

    pub fn on_key_event_playlist_list(&self, key: KeyEvent) {
        let len = self.playlists.lock().unwrap().len();
        let is_renaming = self.renaming.load(Ordering::Relaxed);

        if !is_renaming {
            match key.code {
                KeyCode::Up => {
                    let _ = self.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
                },
                KeyCode::Down => {
                    let _ = self.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
                },
                KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                    self.create_playlist();
                }
                KeyCode::Char('n') if key.modifiers == KeyModifiers::ALT => {
                    self.renaming.store(true, Ordering::Relaxed);
                }
                _ => {},
            }
        } else {
            match key.code {
                KeyCode::Char(char) => {
                    self.selected_playlist_mut(move |pl| {
                        if pl.name.len() < 60 {
                            pl.name.push(char);
                        }
                    });
                }
                KeyCode::Backspace => {
                    self.selected_playlist_mut(move |pl| {
                        pl.name.pop();
                    });
                }
                KeyCode::Esc => {
                    self.renaming.store(false, Ordering::Relaxed);
                }
                KeyCode::Enter => {
                    self.renaming.store(false, Ordering::Relaxed);
                }
                _ => {},
            }
        }
    }

    pub fn on_key_event_song_list(&self, key: KeyEvent) {
        let Some(len) = self.selected_playlist(|pl| pl.songs.len()) else { return };

        match key.code {
            KeyCode::Up => {
                let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
            },
            KeyCode::Down => {
                let _ = self.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
            },
            KeyCode::Enter | KeyCode::Char(_) => {
                let selected_song = self.selected_playlist(|pl| pl.songs[self.selected_song_index.load(Ordering::Relaxed)].clone());
                // log::debug!("selected_song {:?}", selected_song);

                if let Some(song) = selected_song {
                    (self.on_select_fn.lock().unwrap())((song, key));
                }

            },
            _ => {},
        }
    }

}

impl<'a> Widget for Playlists<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl<'a> WidgetRef for Playlists<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_left, _, area_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
            .horizontal_margin(2)
            .areas(area);

        let playlists = self.playlists.lock().unwrap();

        if playlists.len() < 1 {
            return;
        }

        let selected_playlist_index = self.selected_playlist_index.load(Ordering::Relaxed);
        let selected_song = self.selected_song_index.load(Ordering::Relaxed);
        let focused_element = self.focused_element.lock().unwrap();
        let is_renaming = self.renaming.load(Ordering::Relaxed);

        for i in 0..playlists.len() {
            let playlist = &playlists[i];
            let area = Rect {
                y: area_left.y + i as u16,
                height: 1,
                ..area_left
            };

            let style = if i == selected_playlist_index {
                if *focused_element == PlaylistScreenElement::PlaylistList {
                    if is_renaming {
                        Style::default().fg(self.theme.highlight_foreground).bg(Color::Red)
                    } else {
                        Style::default().fg(self.theme.highlight_foreground).bg(self.theme.highlight_background)
                    }
                } else {
                    Style::default().fg(self.theme.highlight_foreground).bg(Color::from_hsl(29.0, 54.0, 34.0))
                }
            } else {
                Style::default().fg(Color::White).bg(self.theme.background)
            };

            let line = if is_renaming && i == selected_playlist_index {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                let caret = if now % 500 < 250 {
                    '⎸'
                } else {
                    ' '
                };
                format!("{}{}", playlist.name, caret)
            } else {
                playlist.name.clone()
            };

            let line = ratatui::text::Line::from(line).style(style);

            line.render_ref(area, buf);
        }

        let selected_playlist = &playlists[selected_playlist_index];

        for i in 0..selected_playlist.songs.len() {
            let song = &selected_playlist.songs[i];
            let area = Rect {
                y: area_right.y + i as u16,
                height: 1,
                ..area_right
            };

            let style = if i == selected_song {
                if *focused_element == PlaylistScreenElement::SongList {
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
    }
}
