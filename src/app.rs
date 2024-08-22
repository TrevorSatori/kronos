use std::error::Error;
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::{env, fs::File, io, path::PathBuf, thread, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Style,
    widgets::Block,
    Frame,
};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::{
    config::Config,
    file_browser::Browser,
    helpers::{
        gen_funcs::{scan_and_filter_directory, Song},
        music_handler::ExtendedSink,
        queue::Queue,
        stateful_list::StatefulList,
        stateful_table::StatefulTable,
    },
    state::State,
    term::set_terminal,
    ui, Command,
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
    player_command_receiver: Arc<Mutex<Receiver<Command>>>,
}

impl<'a> App<'a> {
    pub fn new(
        initial_directory: Option<String>,
        queue: Vec<String>,
        player_command_receiver: Receiver<Command>,
    ) -> Self {
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
            browser: Browser::new(browser_items, current_directory),
            queue_items: Queue::new(queue),
            control_table: StatefulTable::new(),
            player_command_receiver: Arc::new(Mutex::new(player_command_receiver)),
        }
    }

    fn to_state(&self) -> State {
        let queue_items = self
            .queue_items
            .paths()
            .iter()
            .filter_map(|i| i.to_str())
            .map(|i| i.to_string())
            .collect();

        State {
            last_visited_path: self.browser.current_directory.to_str().map(String::from),
            queue_items,
        }
    }

    fn play_pause_recv(&self) {
        let player_command_receiver = self.player_command_receiver.clone();
        let sink = self.sink.clone();

        thread::spawn(move || loop {
            match player_command_receiver.lock().unwrap().recv() {
                Ok(Command::PlayPause) => {
                    sink.toggle();
                }
                Ok(Command::Next) => {
                    sink.stop();
                }
                Err(err) => {
                    eprintln!("error receiving! {}", err);
                    break;
                }
            }
        });
    }

    pub fn start(&mut self) -> Result<State, Box<dyn Error>> {
        let mut terminal = set_terminal()?;

        let tick_rate = Duration::from_secs(1);
        let mut last_tick = std::time::Instant::now();

        self.play_pause_recv();

        loop {
            terminal.draw(|frame| self.render(frame))?;

            self.player_auto_play(); // Up to `tick_rate` lag. A sync channel may be a better alternative.

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key);
                }
            }

            if self.must_quit {
                break;
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = std::time::Instant::now();
            }
        }

        Ok(self.to_state())
    }

    fn set_input_mode(&mut self, in_mode: InputMode) {
        self.input_mode = in_mode
    }

    fn player_sink(&self) -> Arc<Sink> {
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

    pub fn player_auto_play(&mut self) {
        if self.sink.empty() && !self.queue_items.is_empty() {
            let song = self.queue_items.pop();
            self.player_play(song);
        }
    }

    fn player_seek_forward(&mut self) {
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

    fn player_seek_backward(&mut self) {
        let target = self
            .sink
            .get_pos()
            .saturating_sub(Duration::from_secs(5))
            .max(Duration::from_secs(0));
        self.sink.try_seek(target).unwrap_or_else(|e| {
            eprintln!("could not seek {:?}", e);
        });
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
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
            }
            KeyCode::Char('2') => {
                self.active_tab = AppTab::Help;
                self.set_input_mode(InputMode::HelpControls);
            }
            KeyCode::Tab if self.browser.filter.is_none() => {
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
                            }
                            InputMode::Queue => {
                                self.browser.items.unselect();
                                self.queue_items.select_next();
                            }
                            _ => {}
                        };
                    }
                    _ => {}
                }
            }
            KeyCode::Right => self.player_seek_forward(),
            KeyCode::Left => self.player_seek_backward(),
            KeyCode::Char('-') => self.sink.change_volume(-0.05),
            KeyCode::Char('+') => self.sink.change_volume(0.05),
            KeyCode::Char('p') | KeyCode::Char(' ') => self.sink.toggle(),
            KeyCode::Char('g') => self.sink.stop(),
            _ => {
                handled = false;
            }
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

    fn render(self: &mut Self, frame: &mut Frame) {
        let config = Config::new();
        let area = frame.size();

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(area);

        let block = Block::default().style(Style::default().bg(config.background()));
        frame.render_widget(block, area);

        ui::render_ui::render_top_bar(frame, &config, areas[0], self.active_tab);

        match self.active_tab {
            AppTab::FileBrowser => self.browser.render(frame, &self.queue_items, areas[1], &config),
            AppTab::Help => ui::instructions_tab::instructions_tab(frame, &mut self.control_table, areas[1], &config),
        };

        ui::render_ui::render_playing_gauge(
            frame,
            &config,
            areas[2],
            &self.currently_playing.clone(),
            self.player_sink().get_pos(),
            self.queue_items.total_time(),
            self.queue_items.length(),
        );
    }
}
