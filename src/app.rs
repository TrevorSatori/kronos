use std::sync::{Arc};
use std::time::Duration;
use std::{env, io, path::{Path, PathBuf}, thread};
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
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

#[derive(Clone, Copy)]
pub enum InputMode {
    Browser,
    BrowserFilter,
    Queue,
    Controls,
}

#[derive(Debug, Clone, Copy)]
pub enum AppTab {
    Music = 0,
    Controls,
}

impl AppTab {
    pub fn next(&self) -> Self {
        match self {
            Self::Music => Self::Controls,
            Self::Controls => Self::Music,
        }
    }
}

pub struct App<'a> {
    pub browser_items: StatefulList<String>,
    pub queue_items: Queue,
    pub control_table: StatefulTable<'a>,
    music_output: (OutputStream, OutputStreamHandle),
    input_mode: InputMode,
    pub active_tab: AppTab,
    pub last_visited_path: PathBuf,
    pub browser_filter: Option<String>,
    pub must_quit: bool,
    sink: Arc<Sink>,
    pub currently_playing: Option<Song>,
}

impl<'a> App<'a> {
    pub fn new(initial_directory: Option<String>, queue: Vec<String>) -> Self {
        if let Some(path) = initial_directory {
            env::set_current_dir(&path).unwrap_or_else(|err| {
                eprintln!(
                    "Could not set_current_dir to last_visited_path\n\tPath: {}\n\tError: {:?}",
                    path, err
                );
            });
        }

        let mut browser_items = StatefulList::with_items(scan_and_filter_directory());
        browser_items.select(0);

        let music_output = OutputStream::try_default().unwrap();
        let sink = Arc::new(Sink::try_new(&music_output.1).unwrap());

        Self {
            music_output,
            sink,
            browser_items,
            must_quit: false,
            queue_items: Queue::new(queue),
            control_table: StatefulTable::new(),
            input_mode: InputMode::Browser,
            active_tab: AppTab::Music,
            last_visited_path: env::current_dir().unwrap(),
            browser_filter: None,
            currently_playing: None,
        }
    }

    pub fn sink(&self) -> Arc<Sink> {
        self.sink.clone()
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
            last_visited_path: self.last_visited_path.to_str().map(String::from),
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

            self.auto_play(); // Up to `tick_rate` lag. A thread may be a better alternative.

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

    pub fn next(&mut self) {
        self.active_tab = self.active_tab.next();
    }

    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    pub fn set_input_mode(&mut self, in_mode: InputMode) {
        self.input_mode = in_mode
    }

    fn play(&mut self, song: Song) {
        self.sink.stop();

        let path = song.path.clone();
        let sink = self.sink.clone();

        self.currently_playing = Some(song);

        thread::spawn(move || {
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();

            sink.append(source);
            sink.sleep_until_end();
            // TODO: let (tx, rx) = channel(); (see sink.sleep_until_end implementation)
        });
    }

    pub fn current_song(&self) -> Option<Song> {
        if self.sink.empty() && self.queue_items.is_empty() {
            None
        } else {
            self.currently_playing.clone()
        }
    }

    pub fn evaluate(&mut self) {
        let path = self.get_selected_browser_item();

        if path.is_dir() {
            self.last_visited_path = path.clone();
            env::set_current_dir(path).unwrap();
            self.browser_items = StatefulList::with_items(scan_and_filter_directory());
            self.browser_items.next();
        } else {
            self.play(path_to_song(path));
        }
    }

    pub fn backpedal(&mut self) {
        env::set_current_dir("../").unwrap();
        self.browser_items = StatefulList::with_items(scan_and_filter_directory());
        self.browser_items.select_by_path(&self.last_visited_path);
        self.last_visited_path = env::current_dir().unwrap();
    }

    /// Automatically start playing next song if current one has ended.
    pub fn auto_play(&mut self) {
        if self.sink.empty() && !self.queue_items.is_empty() {
            // thread::sleep(Duration::from_millis(250)); // this introduces a pause between tracks. should be configurable, and there must be a better way.
            let song = self.queue_items.pop();
            self.play(song);
        }
    }

    pub fn get_selected_browser_item(&self) -> PathBuf {
        let current_dir = env::current_dir().unwrap();
        if self.browser_items.empty() {
            Path::new(&current_dir).into()
        } else {
            Path::join(&current_dir, Path::new(&self.browser_items.item()))
        }
    }

    fn select_next_browser_by_match(&mut self) {
        if let Some(s) = &self.browser_filter {
            self.browser_items.select_next_by_match(s)
        }
    }

    fn select_previous_browser_by_match(&mut self) {
        if let Some(s) = &self.browser_filter {
            self.browser_items.select_previous_by_match(s)
        }
    }

    pub fn seek_forward(&mut self) {
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

    pub fn seek_backward(&mut self) {
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
        match self.input_mode {
            InputMode::Browser => self.handle_browser_key_events(key),
            InputMode::Queue => self.handle_queue_key_events(key),
            InputMode::Controls => self.handle_help_key_events(key),
            InputMode::BrowserFilter => self.handle_browser_filter_key_events(key),
        }
    }

    fn handle_browser_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.must_quit = true;
            }
            KeyCode::Char('p') | KeyCode::Char(' ') => self.sink.toggle(),
            KeyCode::Char('g') => self.sink.stop(),
            KeyCode::Char('a') => {
                self.queue_items.add(self.get_selected_browser_item());
                self.browser_items.next();
            }
            KeyCode::Enter => self.evaluate(),
            KeyCode::Backspace => self.backpedal(),
            KeyCode::Down | KeyCode::Char('j') => self.browser_items.next(),
            KeyCode::Up | KeyCode::Char('k') => self.browser_items.previous(),
            KeyCode::PageUp => self.browser_items.previous_by(5),
            KeyCode::PageDown => self.browser_items.next_by(5),
            KeyCode::End => self
                .browser_items
                .select(self.browser_items.items().len() - 1),
            KeyCode::Home => self.browser_items.select(0),
            KeyCode::Tab => {
                self.browser_items.unselect();
                self.set_input_mode(InputMode::Queue);
                self.queue_items.select_next();
            }
            KeyCode::Right => self.seek_forward(),
            KeyCode::Left => self.seek_backward(),
            KeyCode::Char('-') => self.sink.change_volume(-0.05),
            KeyCode::Char('+') => self.sink.change_volume(0.05),
            KeyCode::Char('2') => {
                self.next();
                match self.input_mode {
                    InputMode::Controls => self.set_input_mode(InputMode::Browser),
                    _ => self.set_input_mode(InputMode::Controls),
                };
            }
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                self.set_input_mode(InputMode::BrowserFilter);
            }
            _ => {}
        }
    }

    fn handle_browser_filter_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.set_input_mode(InputMode::Browser);
                self.browser_filter = None;
            }
            KeyCode::Enter => {
                self.set_input_mode(InputMode::Browser);
                self.browser_filter = None;
                self.evaluate();
            }
            KeyCode::Down => {
                self.select_next_browser_by_match();
            }
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                self.select_next_browser_by_match();
            }
            KeyCode::Up => {
                self.select_previous_browser_by_match();
            }
            KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => {
                self.select_previous_browser_by_match();
            }
            KeyCode::Backspace => {
                self.browser_filter = match &self.browser_filter {
                    Some(s) if s.len() > 0 => Some(s[..s.len() - 1].to_string()), // TODO: s[..s.len()-1] can panic! use .substring crate
                    _ => None,
                };
            }
            KeyCode::Char(char) => {
                self.browser_filter = match &self.browser_filter {
                    Some(s) => Some(s.to_owned() + char.to_string().as_str()),
                    _ => Some(char.to_string()),
                };
                if !self
                    .browser_items
                    .item()
                    .to_lowercase()
                    .contains(&self.browser_filter.clone().unwrap().to_lowercase())
                {
                    self.browser_items
                        .select_next_by_match(&self.browser_filter.clone().unwrap());
                }
            }
            _ => {}
        }
    }

    fn handle_queue_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.must_quit = true,
            KeyCode::Char('p') => self.sink.toggle(),
            KeyCode::Char('g') => self.sink.stop(),
            KeyCode::Enter => {
                if let Some(song) = self.queue_items.selected_song() {
                    self.play(song);
                };
            }
            KeyCode::Down | KeyCode::Char('j') => self.queue_items.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.queue_items.select_previous(),
            KeyCode::Delete => self.queue_items.remove_selected(),
            KeyCode::Right => self.seek_forward(),
            KeyCode::Left => self.seek_backward(),
            KeyCode::Tab => {
                self.queue_items.select_none();
                self.set_input_mode(InputMode::Browser);
                self.browser_items.next();
            }
            KeyCode::Char('2') => {
                self.next();
                match self.input_mode {
                    InputMode::Controls => self.set_input_mode(InputMode::Browser),
                    _ => self.set_input_mode(InputMode::Controls),
                };
            }
            _ => {}
        }
    }

    fn handle_help_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.must_quit = true,
            KeyCode::Char('p') => self.sink.toggle(),
            KeyCode::Char('g') => self.sink.stop(),
            KeyCode::Down | KeyCode::Char('j') => self.control_table.next(),
            KeyCode::Up | KeyCode::Char('k') => self.control_table.previous(),
            KeyCode::Char('1') => {
                self.next();
                match self.input_mode {
                    InputMode::Controls => self.set_input_mode(InputMode::Browser),
                    _ => self.set_input_mode(InputMode::Controls),
                };
            }
            _ => {}
        }
    }
}
