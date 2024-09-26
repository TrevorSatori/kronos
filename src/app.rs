use std::error::Error;
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::{env, path::PathBuf, thread, time::Duration};
use std::io::BufRead;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use log::error;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Style,
    widgets::Block,
    Frame,
};
use rodio::OutputStream;

use crate::{
    config::Config,
    file_browser::{Browser, FileBrowserSelection},
    player::Player,
    state::State,
    term::set_terminal,
    ui,
    ui::KeyboardHandler,
    Command,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum FocusedElement {
    Browser,
    Queue,
    HelpControls,
    Playlists,
    Library,
}

#[derive(Debug, Clone, Copy)]
pub enum AppTab {
    Library = 0,
    Playlists,
    FileBrowser,
    Help,
}

pub struct App<'a> {
    must_quit: bool,
    config: Config,
    focused_element: FocusedElement,
    active_tab: AppTab,
    browser: Browser<'a>,
    help_tab: ui::HelpTab<'a>,
    player_command_receiver: Arc<Mutex<Receiver<Command>>>,
    player: Arc<Player>,
    _music_output: OutputStream,
    playlist: Arc<ui::Playlists<'a>>,
    library: Arc<ui::Library<'a>>,
}

impl<'a> App<'a> {
    pub fn new(player_command_receiver: Receiver<Command>) -> Self {
        let config = Config::from_file();
        let state = State::from_file();

        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap(); // Indirectly this spawns the cpal_alsa_out thread, and creates the mixer tied to it

        let player = Arc::new(Player::new(state.queue_items, output_stream_handle));

        let current_directory = match &state.last_visited_path {
            Some(s) => PathBuf::from(s),
            None => env::current_dir().unwrap(),
        };

        let library = Arc::new(ui::Library::new(config.theme, vec![]));
        library.on_select({
            let player = player.clone();
            move |(song, key)| {
                if key.code == KeyCode::Enter {
                    player.play_song(song);
                } else if key.code == KeyCode::Char('a') {
                    player.enqueue_song(song);
                }
            }
        });

        let playlist = Arc::new(ui::Playlists::new(config.theme, state.playlists));
        playlist.on_select({
            let player = player.clone();
            move |(song, key)| {
                if key.code == KeyCode::Enter {
                    player.play_song(song);
                } else if key.code == KeyCode::Char('a') {
                    player.enqueue_song(song);
                }
            }
        });

        let mut browser = Browser::new(current_directory);
        browser.on_select({
            let player = player.clone();
            let playlists = playlist.clone();
            let media_library = Arc::clone(&library);

            move |(s, key_event)| {
                Self::on_file_browser_key(player.as_ref(), playlists.as_ref(), media_library.as_ref(), s, key_event);
            }
        });

        Self {
            _music_output: output_stream,
            must_quit: false,
            config,
            focused_element: FocusedElement::Library,
            active_tab: AppTab::Library,
            browser,
            help_tab: ui::HelpTab::new(config),
            player_command_receiver: Arc::new(Mutex::new(player_command_receiver)),
            player,
            playlist,
            library,
        }
    }

    fn to_state(&self) -> State {
        let queue_items = self.player.queue().songs().clone();
        let playlists = self.playlist.playlists();

        State {
            last_visited_path: self.browser.current_directory().to_str().map(String::from),
            queue_items: Vec::from(queue_items),
            playlists,
        }
    }

    // Starts the player loop. Blocking.
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let mut terminal = set_terminal()?;

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = std::time::Instant::now();

        self.spawn_media_key_receiver_thread();
        self.player.spawn();

        while !self.must_quit {
            terminal.draw(|frame| self.render(frame))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key);
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = std::time::Instant::now();
            }
        }

        self.to_state().to_file()?;

        Ok(())
    }

    fn spawn_media_key_receiver_thread(&self) {
        let player_command_receiver = self.player_command_receiver.clone();
        let player = self.player.clone();

        thread::Builder::new().name("media_key_receiver".to_string()).spawn(move || loop {
            match player_command_receiver.lock().unwrap().recv() {
                Ok(Command::PlayPause) => {
                    player.toggle();
                }
                Ok(Command::Next) => {
                    player.stop();
                }
                Err(err) => {
                    error!("error receiving! {}", err);
                    break;
                }
            }
        }).unwrap();
    }

    fn on_file_browser_key(
        player: &Player,
        playlists: &ui::Playlists,
        media_library: &ui::Library,
        file_browser_selection: FileBrowserSelection,
        key_event: KeyEvent,
    ) {
        log::debug!("browser.on_select({:?}, {:?})", file_browser_selection, key_event);
        match (file_browser_selection, key_event.code) {
            (FileBrowserSelection::Song(song), KeyCode::Enter) => {
                player.play_song(song);
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Enter) => {
                player.enqueue_cue(cue_sheet);
            }
            (FileBrowserSelection::Song(song), KeyCode::Char('j')) => {
                log::debug!("TODO: browser.on_select(Song({}), j)", song.title);
                media_library.add_song(song.clone());
            }
            (FileBrowserSelection::Song(song), KeyCode::Char('a')) => {
                player.enqueue_song(song);
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Char('a')) => {
                player.enqueue_cue(cue_sheet);
            }
            (FileBrowserSelection::Directory(path), KeyCode::Char('a')) => {
                log::debug!("TODO: browser.on_select(Directory({}), a)", path.display());
                // directory_to_songs_and_folders
            }
            (FileBrowserSelection::Song(song), KeyCode::Char('y')) => {
                playlists.add_song(song);
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Char('y')) => {
                playlists.add_cue(cue_sheet);
            }
            (FileBrowserSelection::Directory(path), KeyCode::Char('y')) => {
                log::debug!("TODO: browser.on_select(Directory({}), y)", path.display());
                // directory_to_songs_and_folders
            }
            (FileBrowserSelection::Directory(path), KeyCode::Char('j')) => {
                log::debug!("TODO: browser.on_select(Directory({}), j)", path.display());
                // let songs = path_to
                // media_library.lock().unwrap().push(song.clone());
                // directory_to_songs_and_folders
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        let focus_trapped = self.focused_element == FocusedElement::Browser && self.browser.filter().is_some();

        if !focus_trapped {
            if self.on_key_mut(key) || self.on_key(key) {
                return;
            }
        };

        let target: Option<&dyn KeyboardHandler> = match self.focused_element {
            FocusedElement::Library => Some(&*self.library),
            FocusedElement::Playlists => Some(&*self.playlist),
            FocusedElement::Queue => Some(&*self.player),
            FocusedElement::HelpControls => Some(&self.help_tab),
            _ => None,
        };

        if let Some(target) = target {
            target.on_key(key);
        } else {
            self.browser.on_key_event(key);
        }
    }

    fn on_key_mut(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => {
                self.must_quit = true;
            }
            KeyCode::Char('1') => {
                self.active_tab = AppTab::Library;
                self.focused_element = FocusedElement::Library;
            }
            KeyCode::Char('2') => {
                self.active_tab = AppTab::Playlists;
                self.focused_element = FocusedElement::Playlists;
            }
            KeyCode::Char('3') => {
                self.active_tab = AppTab::FileBrowser;
                self.focused_element = FocusedElement::Browser;
            }
            KeyCode::Char('4') => {
                self.active_tab = AppTab::Help;
                self.focused_element = FocusedElement::HelpControls;
            }
            KeyCode::Tab if self.browser.filter().is_none() => {
                match self.active_tab {
                    AppTab::FileBrowser => {
                        self.focused_element = match self.focused_element {
                            FocusedElement::Browser => FocusedElement::Queue,
                            FocusedElement::Queue => FocusedElement::Browser,
                            e => e,
                        };

                        // TODO: focus/blur colors
                        match self.focused_element {
                            FocusedElement::Browser => {
                                self.browser.focus();
                                self.player.queue().select_none();
                            }
                            FocusedElement::Queue => {
                                self.browser.blur();
                                self.player.queue().select_next();
                            }
                            _ => {}
                        };
                    }
                    _ => {
                        return false;
                    }
                }
            }
            _ => {
                return false;
            }
        }
        true
    }

    fn spawn_terminal(&self) {
        let cwd = self.browser.current_directory().clone();

        if let Err(err) = thread::Builder::new().name("term".to_string()).spawn(move || {
            log::debug!("spawning child process");

            let proc = std::process::Command::new("kitty")
                .current_dir(cwd)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            if let Ok(mut proc) = proc {
                log::debug!("spawned child process");

                let stdout = proc.stdout.as_mut().unwrap();
                let stdout_reader = std::io::BufReader::new(stdout);

                for line in stdout_reader.lines() {
                    log::debug!("stdout: {:?}", line);
                }

                log::debug!("child process exited");
            } else if let Err(err) = proc {
                log::error!("error spawning thread {:?}", err);
            }
        }) {
            log::error!("Error spawning thread! {:?}", err);
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let block = Block::default().style(Style::default().bg(self.config.theme.background));
        frame.render_widget(block, frame.size());

        let [area_top, _, area_center, area_bottom] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0), Constraint::Length(3)]).areas(frame.size());

        ui::render_top_bar(frame, &self.config, area_top, self.active_tab);

        match self.active_tab {
            AppTab::Library => {
                frame.render_widget(&*self.library, area_center);
            },
            AppTab::Playlists => {
                frame.render_widget(&*self.playlist, area_center); // &*...? Is this ok?
            },
            AppTab::FileBrowser => self.browser.render(frame, &self.player.queue(), area_center, &self.config),
            AppTab::Help => self.help_tab.render(frame, area_center),
        };

        let currently_playing = self.player.currently_playing();
        let currently_playing = currently_playing.lock().unwrap();

        ui::render_playing_gauge(
            frame,
            &self.config,
            area_bottom,
            &currently_playing,
            self.player.get_pos(),
            self.player.queue().total_time(),
            self.player.queue().length(),
        );
    }
}

impl<'a> KeyboardHandler for App<'a> {
    fn on_key(&self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Right => self.player.seek_forward(),
            KeyCode::Left => self.player.seek_backward(),
            KeyCode::Char('-') => self.player.change_volume(-0.05),
            KeyCode::Char('+') => self.player.change_volume(0.05),
            KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => self.player.toggle(),
            KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => self.player.stop(),
            KeyCode::Char('c') if key.modifiers == KeyModifiers::ALT => self.spawn_terminal(),
            _ => {
                return false;
            }
        }
        true
    }
}
