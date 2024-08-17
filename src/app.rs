use std::{env, io, path::{Path, PathBuf}, thread, time::Duration, fs::File};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::Receiver};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::{
    config::Config,
    helpers::{
        gen_funcs::{path_to_song, scan_and_filter_directory, Song},
        music_handler::{ExtendedSink},
        queue::Queue,
        stateful_list::StatefulList,
        stateful_table::StatefulTable,
    },
    state::State,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InputMode {
    Browser,
    Queue,
    HelpControls,
}

#[derive(Debug, Clone, Copy)]
pub enum AppTab {
    FileBrowser = 0,
    Help,
}

pub struct Browser {
    pub items: StatefulList<String>,
    pub current_directory: PathBuf,
    pub filter: Option<String>,
}

impl Browser {
    pub fn selected_item(&self) -> PathBuf {
        if self.items.empty() {
            Path::new(&self.current_directory).into()
        } else {
            Path::join(&self.current_directory, Path::new(&self.items.item()))
        }
    }

    pub fn enter_selection(&mut self) -> Option<Song> {
        let path = self.selected_item();

        if path.is_dir() {
            self.navigate_into();
            None
        } else {
            match path_to_song(&path) {
                Ok(song) => Some(song),
                Err(err) => None,
            }
        }
    }

    pub fn navigate_into(&mut self) {
        let path = self.selected_item();

        if path.is_dir() {
            self.current_directory = path.clone();
            self.items = StatefulList::with_items(scan_and_filter_directory(&path));
            self.items.next();
        }
    }

    pub fn navigate_up(&mut self) {
        let parent = self.current_directory.as_path().parent().unwrap().to_path_buf();
        self.items = StatefulList::with_items(scan_and_filter_directory(&parent));
        self.items.select_by_path(&self.current_directory);
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

    fn on_key_event(&mut self, key: KeyEvent) {
        if !self.filter.is_some() {
            self.on_normal_key_event(key);
        } else  {
            self.on_filter_key_event(key);
        }
    }

    fn on_normal_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Backspace => self.navigate_up(),
            KeyCode::Down | KeyCode::Char('j') => self.items.next(),
            KeyCode::Up | KeyCode::Char('k') => self.items.previous(),
            KeyCode::PageUp => self.items.previous_by(5),
            KeyCode::PageDown => self.items.next_by(5),
            KeyCode::End => self.select_last(),
            KeyCode::Home => self.items.select(0),
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                self.filter = Some("".to_string());
            }
            _ => {}
        }
    }

    fn on_filter_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.filter = None;
            }
            KeyCode::Esc => {
                self.filter = None;
            }
            KeyCode::Down => {
                self.select_next_match();
            }
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                self.select_next_match();
            }
            KeyCode::Up => {
                self.select_previous_match();
            }
            KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => {
                self.select_previous_match();
            }
            KeyCode::Backspace => {
                self.filter_delete();
            }
            KeyCode::Char(char) => {
                self.filter_append(char);
            }
            _ => {}
        }
    }
}

pub struct App<'a> {
    must_quit: bool,
    input_mode: InputMode,
    active_tab: AppTab,
    pub browser: Browser,
    pub queue_items: Queue,
    pub control_table: StatefulTable<'a>,
    music_output: (OutputStream, OutputStreamHandle),
    sink: Arc<Sink>,
    currently_playing: Option<Song>,
}

impl<'a> App<'a> {
    pub fn new(initial_directory: Option<String>, queue: Vec<String>) -> Self {
        let current_directory = match &initial_directory {
            Some(s) => PathBuf::from(s),
            None => env::current_dir().unwrap(),
        };

        let mut browser_items = StatefulList::with_items(scan_and_filter_directory(&current_directory));
        browser_items.select(0);

        let music_output = OutputStream::try_default().unwrap();
        let sink = Arc::new(Sink::try_new(&music_output.1).unwrap());

        Self {
            must_quit: false,
            input_mode: InputMode::Browser,
            active_tab: AppTab::FileBrowser,
            music_output,
            sink,
            currently_playing: None,
            browser: Browser {
                items: browser_items,
                current_directory,
                filter: None,
            },
            queue_items: Queue::new(queue),
            control_table: StatefulTable::new(),
        }
    }

    fn to_state(&self) -> State {
        let queue_items = self
            .queue_items
            .paths()
            .iter()
            .map(|i| i.to_str())
            .filter(|i| i.is_some())
            .map(|i| i.unwrap().to_string())
            .collect();

        State {
            last_visited_path: self.browser.current_directory.to_str().map(String::from),
            queue_items,
        }
    }

    fn play_pause_recv(&self, play_pause_receiver: Receiver<()>, sink: Arc<Sink>) {
        thread::spawn(move || {
            loop {
                if let Err(err) = play_pause_receiver.recv() {
                    eprintln!("error receiving! {}", err);
                } else {
                    sink.toggle();
                }
            }
        });
    }

    pub fn start<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        play_pause_receiver: Receiver<()>,
        quit: Arc<AtomicBool>,
    ) -> io::Result<State> {
        let cfg = Config::new();
        let tick_rate = Duration::from_secs(1);
        let mut last_tick = std::time::Instant::now();

        self.play_pause_recv(play_pause_receiver, self.sink.clone());

        loop {
            let currently_playing = &self.currently_playing.clone();
            terminal.draw(|f| crate::ui::render_ui(f, self, &cfg, self.active_tab, currently_playing))?;

            self.player_auto_play(); // Up to `tick_rate` lag. A thread may be a better alternative.

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key);
                }
            }

            if self.must_quit {
                quit.store(true, Ordering::Relaxed);
                break;
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = std::time::Instant::now();
            }
        }

        Ok(self.to_state())
    }

    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    pub fn set_input_mode(&mut self, in_mode: InputMode) {
        self.input_mode = in_mode
    }

    pub fn player_sink(&self) -> Arc<Sink> {
        self.sink.clone()
    }

    fn player_play(&mut self, song: Song) {
        self.sink.stop();

        let path = song.path.clone();
        let sink = self.sink.clone();

        self.currently_playing = Some(song);

        thread::spawn(move || {
            let file = io::BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();

            sink.append(source);
            sink.sleep_until_end();
            // TODO: let (tx, rx) = channel(); (see sink.sleep_until_end implementation)
        });
    }

    pub fn player_current_song(&self) -> Option<Song> {
        if self.sink.empty() && self.queue_items.is_empty() {
            None
        } else {
            self.currently_playing.clone()
        }
    }

    pub fn player_auto_play(&mut self) {
        if self.sink.empty() && !self.queue_items.is_empty() {
            let song = self.queue_items.pop();
            self.player_play(song);
        }
    }

    pub fn player_seek_forward(&mut self) {
        if let Some(song) = &self.currently_playing {
            let target = self
                .sink
                .get_pos()
                .saturating_add(Duration::from_secs(5))
                .min(song.length);
            self.sink.try_seek(target).unwrap_or_else(|e| {
                eprintln!("could not seek {:?}", e);
            });
        }
    }

    pub fn player_seek_backward(&mut self) {
        let target = self
            .sink
            .get_pos()
            .saturating_sub(Duration::from_secs(5))
            .max(Duration::from_secs(0));
        self.sink.try_seek(target).unwrap_or_else(|e| {
            eprintln!("could not seek {:?}", e);
        });
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let focus_trapped = self.input_mode == InputMode::Browser && self.browser.filter.is_some();
        let handled = !focus_trapped && self.handle_app_key_event(&key);

        if !handled {
            match self.input_mode {
                InputMode::Browser => self.handle_browser_key_events(key),
                InputMode::Queue => self.handle_queue_key_events(key),
                InputMode::HelpControls => self.handle_help_key_events(key),
            }
        }
    }

    fn handle_app_key_event(&mut self, key: &KeyEvent) -> bool {
        let mut handled = true;
        match key.code {
            KeyCode::Char('q') => {
                self.must_quit = true;
            }
            KeyCode::Char('1') => {
                self.active_tab = AppTab::FileBrowser;
                self.set_input_mode(InputMode::Browser);
            },
            KeyCode::Char('2') => {
                self.active_tab = AppTab::Help;
                self.set_input_mode(InputMode::HelpControls);
            },
            KeyCode::Tab => if self.browser.filter.is_none() {
                match self.active_tab {
                    AppTab::FileBrowser => {
                        self.input_mode = match self.input_mode {
                            InputMode::Browser => InputMode::Queue,
                            InputMode::Queue => InputMode::Browser,
                            e => e,
                        };

                        // TODO: focus/blur colors
                        match self.input_mode {
                            InputMode::Browser => {
                                self.browser.items.next();
                                self.queue_items.select_none();
                            },
                            InputMode::Queue => {
                                self.browser.items.unselect();
                                self.queue_items.select_next();
                            },
                            _ => {},
                        };
                    },
                    _ => {},
                }
            }
            KeyCode::Right => self.player_seek_forward(),
            KeyCode::Left => self.player_seek_backward(),
            KeyCode::Char('-') => self.sink.change_volume(-0.05),
            KeyCode::Char('+') => self.sink.change_volume(0.05),
            KeyCode::Char('p') | KeyCode::Char(' ') => self.sink.toggle(),
            KeyCode::Char('g') => self.sink.stop(),
            _ => { handled = false; },
        }
        handled
    }

    fn handle_browser_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter if key.modifiers == KeyModifiers::ALT => {
                self.queue_items.add(self.browser.selected_item());
                self.browser.items.next();
            }
            KeyCode::Char('a') if self.browser.filter.is_none() => {
                self.queue_items.add(self.browser.selected_item());
                self.browser.items.next();
            }
            KeyCode::Enter => {
                if let Some(song) = self.browser.enter_selection() {
                    self.player_play(song);
                }
            }
            _ => {}
        }

        self.browser.on_key_event(key);
    }

    fn handle_queue_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if let Some(song) = self.queue_items.selected_song() {
                    self.player_play(song);
                };
            }
            KeyCode::Down | KeyCode::Char('j') => self.queue_items.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.queue_items.select_previous(),
            KeyCode::Delete => self.queue_items.remove_selected(),
            _ => {}
        }
    }

    fn handle_help_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.control_table.next(),
            KeyCode::Up | KeyCode::Char('k') => self.control_table.previous(),
            _ => {}
        }
    }
}
