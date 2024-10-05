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
    structs::{Song, Playlist},
    config::Theme,
    cue::CueSheet,
    ui::{song_to_string, KeyboardHandlerRef},
};

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
                        Style::default().fg(self.theme.foreground_selected).bg(self.theme.search)
                    } else {
                        Style::default().fg(self.theme.foreground_selected).bg(self.theme.background_selected)
                    }
                } else {
                    Style::default().fg(self.theme.foreground_selected).bg(self.theme.background_selected_blur)
                }
            } else {
                Style::default().fg(self.theme.foreground_secondary).bg(self.theme.background)
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

        if selected_playlist_index >= playlists.len() {
            log::error!("selected_playlist_index >= playlists.len()");
            return;
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
                    Style::default().fg(self.theme.foreground_selected).bg(self.theme.background_selected)
                } else {
                    Style::default().fg(self.theme.foreground_selected).bg(self.theme.background_selected_blur)
                }
            } else {
                Style::default().fg(self.theme.foreground_secondary).bg(self.theme.background)
            };

            let line = ratatui::text::Line::from(song_to_string(song)).style(style);
            line.render_ref(area, buf);
        }
    }
}

impl<'a> KeyboardHandlerRef<'a> for Playlists<'a> {

    fn on_key(&self, key: KeyEvent) -> bool {
        let mut focused_element_guard = self.focused_element.lock().unwrap();

        match key.code {
            KeyCode::Tab => {
                *focused_element_guard = match *focused_element_guard {
                    PlaylistScreenElement::PlaylistList => PlaylistScreenElement::SongList,
                    PlaylistScreenElement::SongList => PlaylistScreenElement::PlaylistList,
                };
            }
            _ if *focused_element_guard == PlaylistScreenElement::PlaylistList  => {
                on_key_event_playlist_list(&self, key);
            },
            _ if *focused_element_guard == PlaylistScreenElement::SongList  => {
                on_key_event_song_list(&self, key);
            },
            _ => {
                return false;
            },
        }
        true
    }

}


fn on_key_event_playlist_list(s: &Playlists, key: KeyEvent) {
    let len = s.playlists.lock().unwrap().len();
    let is_renaming = s.renaming.load(Ordering::Relaxed);

    if !is_renaming {
        match key.code {
            KeyCode::Up => {
                let _ = s.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
            },
            KeyCode::Down => {
                let _ = s.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
            },
            KeyCode::Home => {
                s.selected_playlist_index.store(0, Ordering::Relaxed);
            },
            KeyCode::End => {
                s.selected_playlist_index.store(len.saturating_sub(1), Ordering::Relaxed);
            },
            KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                s.create_playlist();
                let _ = s.selected_playlist_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len)) });
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                s.renaming.store(true, Ordering::Relaxed);
            }
            KeyCode::Delete => {
                let selected_playlist_index = s.selected_playlist_index.load(Ordering::Relaxed);
                let mut playlists = s.playlists.lock().unwrap();

                if playlists.len() > 0 {
                    playlists.remove(selected_playlist_index);
                    if selected_playlist_index > playlists.len().saturating_sub(1) {
                        s.selected_playlist_index.store(playlists.len().saturating_sub(1), Ordering::Relaxed);
                    }
                }
            }
            _ => {},
        }
    } else {
        match key.code {
            KeyCode::Char(char) => {
                s.selected_playlist_mut(move |pl| {
                    if pl.name.len() < 60 {
                        pl.name.push(char);
                    }
                });
            }
            KeyCode::Backspace => {
                s.selected_playlist_mut(move |pl| {
                    if key.modifiers == KeyModifiers::ALT {
                        pl.name.clear();
                    } else {
                        pl.name.pop();
                    }
                });
            }
            KeyCode::Esc => {
                s.renaming.store(false, Ordering::Relaxed);
            }
            KeyCode::Enter => {
                s.renaming.store(false, Ordering::Relaxed);
            }
            _ => {},
        }
    }
}

fn on_key_event_song_list(s: &Playlists, key: KeyEvent) {
    let Some(len) = s.selected_playlist(|pl| pl.songs.len()) else { return };

    match key.code {
        KeyCode::Up if key.modifiers == KeyModifiers::NONE => {
            let _ = s.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_sub(1)) });
        },
        KeyCode::Down if key.modifiers == KeyModifiers::NONE => {
            let _ = s.selected_song_index.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| { Some(a.saturating_add(1).min(len.saturating_sub(1))) });
        },
        KeyCode::Up if key.modifiers == KeyModifiers::ALT => {
            let selected_song = s.selected_song_index.load(Ordering::Relaxed);
            s.selected_playlist_mut(|pl| {
                if pl.songs.len() > 1 && selected_song > 0 {
                    pl.songs.swap(selected_song, selected_song - 1);
                    s.selected_song_index.store(selected_song - 1, Ordering::Relaxed);
                }
            });
        },
        KeyCode::Down if key.modifiers == KeyModifiers::ALT => {
            let selected_song = s.selected_song_index.load(Ordering::Relaxed);
            s.selected_playlist_mut(|pl| {
                if pl.songs.len() > 1 && selected_song < pl.songs.len() - 1 {
                    pl.songs.swap(selected_song, selected_song + 1);
                    s.selected_song_index.store(selected_song + 1, Ordering::Relaxed);
                }
            });
        },
        KeyCode::Enter | KeyCode::Char(_) => {
            let selected_song = s.selected_playlist(|pl| pl.songs[s.selected_song_index.load(Ordering::Relaxed)].clone());
            if let Some(song) = selected_song {
                s.on_select_fn.lock().unwrap()((song, key));
            }
        },
        KeyCode::Delete => {
            let selected_song = s.selected_song_index.load(Ordering::Relaxed);
            s.selected_playlist_mut(|pl| {
                if pl.songs.len() > 0 {
                    pl.songs.remove(selected_song);
                    if selected_song >= pl.songs.len() {
                        s.selected_song_index.store(selected_song.saturating_sub(1), Ordering::Relaxed);
                    }
                }
            });
        },
        _ => {},
    }
}

impl Drop for Playlists<'_> {
    fn drop(&mut self) {
        log::trace!("Playlists.drop()");
    }
}
