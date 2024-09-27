use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::error;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::{Line, Modifier, Span, Style},
    style::Color,
    text::Text,
    widgets::{
        block::Position,
        Block,
        BorderType,
        Borders,
        List,
        WidgetRef,
        StatefulWidget,
        StatefulWidgetRef,
    },
};

use crate::{
    cue::CueSheet,
    structs::{Song, Queue},
    ui,
    ui::KeyboardHandlerMut,
    ui::stateful_list::StatefulList,
    config::{Theme},

};

const VALID_EXTENSIONS: [&str; 8] = ["mp3", "mp4", "m4a", "wav", "flac", "ogg", "aac", "cue"];

pub struct Browser<'a> {
    theme: Theme,
    items: StatefulList<String>,
    current_directory: PathBuf,
    filter: Option<String>,
    last_offset: usize,
    on_select_fn: Box<dyn FnMut((FileBrowserSelection, KeyEvent)) + 'a>,
    queue_items: Arc<Queue>,
}

#[allow(dead_code, unused_variables)]
#[derive(Debug)]
pub enum FileBrowserSelection {
    Song(Song),
    CueSheet(CueSheet),
    Directory(PathBuf),
}

fn directory_to_songs_and_folders(path: &PathBuf) -> Vec<String> {
    // TODO: .cue
    let Ok(entries) = path.read_dir() else {
        return vec![];
    };

    let mut items: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|entry| dir_entry_is_dir(&entry) || dir_entry_is_song(&entry))
        .map(|entry| entry.path())
        .filter(path_is_not_hidden)
        .filter_map(|path| path.file_name().and_then(|e| e.to_str()).map(|e| e.to_string()))
        .collect();

    items.sort_unstable();
    items
}

fn dir_entry_is_file(dir_entry: &DirEntry) -> bool {
    // TODO: resolve symlinks
    dir_entry.file_type().is_ok_and(|ft| ft.is_file())
}

fn dir_entry_is_dir(dir_entry: &DirEntry) -> bool {
    let Ok(ft) = dir_entry.file_type() else {
        log::error!("dir_entry_is_dir: .file_type() returned error for {:?}", dir_entry.path());
        return false;
    };

    if ft.is_symlink() {
        let ln = fs::canonicalize(dir_entry.path());
        ln.is_ok_and(|ln| ln.is_dir())
    } else {
        ft.is_dir()
    }
}

fn path_is_not_hidden(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|e| e.to_str())
        .map(|e| e.to_string())
        .is_some_and(|d| !d.starts_with('.'))
}

fn dir_entry_has_song_extension(dir_entry: &DirEntry) -> bool {
    dir_entry
        .path()
        .extension()
        .is_some_and(|e| VALID_EXTENSIONS.contains(&e.to_str().unwrap()))
}

fn dir_entry_is_song(dir_entry: &DirEntry) -> bool {
    dir_entry_is_file(dir_entry) && dir_entry_has_song_extension(dir_entry)
}

impl<'a> Browser<'a> {
    pub fn new(theme: Theme, current_directory: PathBuf, queue_items: Arc<Queue>) -> Self {
        let mut items = StatefulList::with_items(directory_to_songs_and_folders(&current_directory));
        items.select(0);

        Self {
            theme,
            items,
            current_directory,
            filter: None,
            last_offset: 0,
            on_select_fn: Box::new(|_| {}) as _,
            queue_items,
        }
    }

    pub fn items(&self) -> &StatefulList<String> {
        &self.items
    }

    pub fn filter(&self) -> &Option<String> {
        &self.filter
    }

    pub fn blur(&mut self) {
        self.items.unselect();
    }

    pub fn focus(&mut self) {
        self.items.next();
    }

    pub fn current_directory(&self) -> &PathBuf {
        &self.current_directory
    }

    pub fn selected_item(&self) -> PathBuf {
        if self.items.empty() {
            Path::new(&self.current_directory).into()
        } else {
            Path::join(&self.current_directory, Path::new(&self.items.item()))
        }
    }

    pub fn on_select(&mut self, cb: impl FnMut((FileBrowserSelection, KeyEvent)) + 'a) {
        self.on_select_fn = Box::new(cb);
    }

    fn enter_selection(&mut self, key_event: KeyEvent) {
        let path = self.selected_item();

        if path.is_dir() {
            if key_event.code == KeyCode::Enter {
                self.navigate_into();
            } else {
                (self.on_select_fn)((FileBrowserSelection::Directory(path), key_event));
            }
        } else if path.extension().is_some_and(|e| e == "cue") {
            match CueSheet::from_file(&path) {
                Ok(cue_sheet) => {
                    (self.on_select_fn)((FileBrowserSelection::CueSheet(cue_sheet), key_event));
                    self.items.next();
                }
                Err(err) => {
                    error!("Filed to read CueSheet {:#?}", err);
                }
            }
        } else {
            match Song::from_file(&path) {
                Ok(song) => {
                    (self.on_select_fn)((FileBrowserSelection::Song(song), key_event));
                    self.items.next();
                }
                Err(err) => {
                    error!("Failed to read Song {:#?}", err);
                }
            }
        }
    }

    pub fn navigate_into(&mut self) {
        let path = self.selected_item();

        if path.is_dir() {
            self.current_directory = path.clone();
            self.last_offset = self.items.offset;
            self.items = StatefulList::with_items(directory_to_songs_and_folders(&path));
            self.items.next();
        }
    }

    pub fn navigate_up(&mut self) {
        let Some(parent) = self.current_directory.as_path().parent().map(|p| p.to_path_buf()) else { return };
        self.items = StatefulList::with_items(directory_to_songs_and_folders(&parent));
        self.items.select_by_path(&self.current_directory);
        self.items.offset = self.last_offset;
        self.current_directory = parent;
    }

    pub fn select_last(&mut self) {
        self.items.select(self.items.items().len() - 1)
    }

    pub fn select_next_match(&mut self) {
        if let Some(s) = &self.filter {
            self.items.select_next_by_match(s)
        }
    }

    pub fn select_previous_match(&mut self) {
        if let Some(s) = &self.filter {
            self.items.select_previous_by_match(s)
        }
    }

    pub fn filter_delete(&mut self) {
        self.filter = match &self.filter {
            Some(s) if s.len() > 0 => Some(s[..s.len() - 1].to_string()), // TODO: s[..s.len()-1] can panic! use .substring crate
            _ => None,
        };
    }

    pub fn filter_append(&mut self, char: char) {
        self.filter = match &self.filter {
            Some(s) => Some(s.to_owned() + char.to_string().as_str()),
            _ => Some(char.to_string()),
        };
        if !self
            .items
            .item()
            .to_lowercase()
            .contains(&self.filter.clone().unwrap().to_lowercase())
        {
            self.items.select_next_by_match(&self.filter.clone().unwrap());
        }
    }

}

impl<'a> KeyboardHandlerMut<'a> for Browser<'a> {
    fn on_key(&mut self, key: KeyEvent) -> bool {
        if !self.filter.is_some() {
            on_normal_key_event(self, key);
        } else {
            on_filter_key_event(self, key);
        }

        true
    }
}

fn on_normal_key_event(browser: &mut Browser, key: KeyEvent) {
    match key.code {
        // KeyCode::Enter => { browser.enter_selection(key); },
        KeyCode::Backspace => browser.navigate_up(),
        KeyCode::Down => browser.items.next(),
        KeyCode::Up => browser.items.previous(),
        KeyCode::PageUp => browser.items.previous_by(5),
        KeyCode::PageDown => browser.items.next_by(5),
        KeyCode::End => browser.select_last(),
        KeyCode::Home => browser.items.select(0),
        KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
            browser.filter = Some("".to_string());
        }
        KeyCode::Enter | KeyCode::Char(_) => {
            browser.enter_selection(key);
        },
        _ => {}
    }
}

fn on_filter_key_event(browser: &mut Browser, key: KeyEvent) {
    match key.code {
        KeyCode::Enter if key.modifiers == KeyModifiers::ALT => {
            browser.enter_selection(key);
        }
        KeyCode::Enter => {
            browser.filter = None;
            browser.enter_selection(key);
        }
        KeyCode::Esc => {
            browser.filter = None;
        }
        KeyCode::Down => {
            browser.select_next_match();
        }
        KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
            browser.select_next_match();
        }
        KeyCode::Up => {
            browser.select_previous_match();
        }
        KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => {
            browser.select_previous_match();
        }
        KeyCode::Backspace => {
            browser.filter_delete();
        }
        KeyCode::Char(char) => {
            browser.filter_append(char);
        }
        _ => {}
    }
}



impl<'a> WidgetRef for &Browser<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let (area_top, area_main_left, area_main_separator, area_main_right) = create_areas(area);

        // self.set_height(area_main_left.height);
        // TODO: not use stateful list

        let tb = top_bar(&self.theme, self.current_directory(), &self.filter);
        tb.render_ref(area_top, buf);

        let fl = file_list(&self.theme, self.items(), &self.filter());
        StatefulWidget::render(
            fl,
            area_main_left,
            buf,
            &mut self.items.state(),
        );

        let [_separator_left, separator_middle, _separator_right] = Layout::horizontal([Constraint::Min(1), Constraint::Length(1), Constraint::Min(1)])
            .areas(area_main_separator);

        let vertical_separator = Block::default().borders(Borders::RIGHT).border_type(BorderType::Double);
        vertical_separator.render_ref(separator_middle, buf);

        let ql = queue_list(&self.theme, &self.queue_items);
        StatefulWidget::render(
            ql,
            area_main_right,
            buf,
            &mut ratatui::widgets::ListState::default().with_selected(self.queue_items.selected_song_index())
        );

    }
}

fn create_areas(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let [area_top, area_main] = Layout::vertical([Constraint::Length(2), Constraint::Min(1)])
        .horizontal_margin(2)
        .areas(area);

    let [area_main_left, area_main_separator, area_main_right] =
        Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
        .areas(area_main);

    (area_top, area_main_left, area_main_separator, area_main_right)
}

fn top_bar(theme: &Theme, current_directory: &Path, filter: &Option<String>) -> Block<'static> {
    let folder_name = current_directory
        .file_name()
        .map(|s| s.to_str())
        .flatten()
        .map(String::from)
        .unwrap_or("".to_string());

    let browser_title = match filter {
        Some(filter) => Line::from(vec![
            Span::styled("Search: ", Style::default()),
            Span::styled(filter.clone(), Style::default().fg(theme.search)),
        ]),
        _ => Line::from(folder_name),
    };

    let top_bar = Block::default()
        .borders(Borders::NONE)
        .title(browser_title)
        .title_alignment(Alignment::Left)
        .title_position(Position::Top)
        .title_style(Style::new().bg(theme.background).fg(theme.foreground));

    top_bar
}

fn file_list(theme: &Theme, items: &StatefulList<String>, filter: &Option<String>) -> List<'static> {
    let browser_items: Vec<ratatui::widgets::ListItem> = items
        .items()
        .iter()
        .map(|i| {
            let fg = match filter.as_ref() {
                Some(s) if i.to_lowercase().contains(&s.to_lowercase()) => theme.search,
                _ => Color::Reset,
            };
            ratatui::widgets::ListItem::new(Text::from(i.to_owned())).style(Style::default().fg(fg))
        })
        .collect();

    let browser_list = List::new(browser_items)
        .style(Style::default().fg(theme.foreground))
        .highlight_style(
            Style::default()
                .bg(theme.highlight_background)
                .fg(theme.highlight_foreground)
                .add_modifier(Modifier::BOLD),
        )
        .scroll_padding(0)
        .highlight_symbol("");

    browser_list
}

fn queue_list<'a>(theme: &Theme, queue_items: &Queue) -> List<'a> {
    let queue_items: Vec<String> = queue_items.songs().iter().map(ui::song_to_string).collect();

    let queue_list = List::new(queue_items)
        .style(Style::default().fg(theme.foreground))
        .highlight_style(
            Style::default()
                .bg(theme.highlight_background)
                .fg(theme.highlight_foreground)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");

    queue_list
}
