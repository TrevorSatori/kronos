use std::error::Error;
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::{env, path::PathBuf, thread, time::Duration};

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
    structs::Song,
    file_browser::{Browser, FileBrowserSelection},
    player::Player,
    state::State,
    term::set_terminal,
    ui,
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
    #[allow(dead_code)]
    music_output: OutputStream,
    playlist: Arc<ui::Playlists<'a>>,
    media_library: Arc<Mutex<Vec<Song>>>,
}

impl<'a> App<'a> {
    pub fn new(player_command_receiver: Receiver<Command>) -> Self {
        let config = Config::from_file();
        let state = State::from_file();

        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap(); // Indirectly this spawns the cpal_alsa_out thread, and creates the mixer tied to it
        // output_stream is !Send + !Sync, and we want Player to be Send+Sync, so
        // App will own it and pass just the weak reference to it.
        // output_stream_handle is roughly a Weak<output_stream> that IS Send+Sync (because it doesn't contain the cpal stream in it)
        // See https://github.com/RustAudio/cpal/blob/bbb58ab76787d090d32ed56964bfcf194b8f6a3d/src/platform/mod.rs#L67
        // Note: if this is only true for Android, it'd be nice to just drop this requirement.

        let player = Arc::new(Player::new(state.queue_items, output_stream_handle));
        let playlist = Arc::new(ui::Playlists::new(config.theme, state.playlists));
        let media_library = Arc::new(Mutex::new(Vec::new()));

        let current_directory = match &state.last_visited_path {
            Some(s) => PathBuf::from(s),
            None => env::current_dir().unwrap(),
        };

        let mut browser = Browser::new(current_directory);
        browser.on_select({
            let player = player.clone();
            let playlists = playlist.clone();
            let media_library = Arc::clone(&media_library);

            move |(s, key_event)| {
                Self::on_file_browser_key(player.as_ref(), playlists.as_ref(), media_library.as_ref(), s, key_event);
            }
        });

        let app = Self {
            music_output: output_stream,
            must_quit: false,
            config,
            focused_element: FocusedElement::Library,
            active_tab: AppTab::Library,
            browser,
            help_tab: ui::HelpTab::new(config),
            player_command_receiver: Arc::new(Mutex::new(player_command_receiver)),
            player,
            playlist,
            media_library,
        };

        app.playlist.on_select({
            let player = app.player.clone();
            move |(song, key)| {
                if key.code == KeyCode::Enter {
                    player.play_song(song);
                } else if key.code == KeyCode::Char('a') {
                    player.enqueue_song(song);
                }
           }
        });

        app
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
        media_library: &Mutex<Vec<Song>>,
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
                media_library.lock().unwrap().push(song.clone());
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
        let handled = !focus_trapped && self.handle_app_key_event(key);

        if !handled {
            match self.focused_element {
                FocusedElement::Library =>{},
                FocusedElement::Playlists => { self.playlist.on_key_event(key) },
                FocusedElement::Browser => self.browser.on_key_event(key),
                FocusedElement::Queue => self.handle_queue_key_events(key),
                FocusedElement::HelpControls => self.handle_help_key_events(key),
            }
        }
    }

    fn handle_app_key_event(&mut self, key: KeyEvent) -> bool {
        let mut handled = true;
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
                    AppTab::Playlists => {
                        self.playlist.on_key_event(key);
                    }
                    _ => {}
                }
            }
            KeyCode::Right => self.player.seek_forward(),
            KeyCode::Left => self.player.seek_backward(),
            KeyCode::Char('-') => self.player.change_volume(-0.05),
            KeyCode::Char('+') => self.player.change_volume(0.05),
            KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => self.player.toggle(),
            KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => self.player.stop(),
            KeyCode::Char('c') if key.modifiers == KeyModifiers::ALT => {
                let _ = env::set_current_dir(self.browser.current_directory());
            }
            _ => {
                handled = false;
            }
        }
        handled
    }

    fn handle_queue_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if let Some(song) = self.player.queue().selected_song() {
                    self.player.play_song(song);
                };
            }
            KeyCode::Down | KeyCode::Char('j') => self.player.queue().select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.player.queue().select_previous(),
            KeyCode::Delete => self.player.queue().remove_selected(),
            _ => {}
        }
    }

    fn handle_help_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.help_tab.next(),
            KeyCode::Up | KeyCode::Char('k') => self.help_tab.previous(),
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let block = Block::default().style(Style::default().bg(self.config.theme.background));
        frame.render_widget(block, frame.size());

        let [area_top, _, area_center, area_bottom] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0), Constraint::Length(3)]).areas(frame.size());

        ui::render_top_bar(frame, &self.config, area_top, self.active_tab);

        match self.active_tab {
            AppTab::Library => {},
            AppTab::Playlists => {
                frame.render_widget(&*self.playlist, area_center); // &*...? Is this ok?
            },
            AppTab::FileBrowser => self.browser.render(frame, &self.player.queue(), area_center, &self.config),
            AppTab::Help => self.help_tab.render(frame, area_center),
        };

        // log::debug!("ui acquiring currently_playing");
        let currently_playing = self.player.currently_playing();
        let currently_playing = currently_playing.lock().unwrap();
        // log::debug!("ui acquired currently_playing");

        ui::render_playing_gauge(
            frame,
            &self.config,
            area_bottom,
            &currently_playing,
            self.player.get_pos(),
            self.player.queue().total_time(),
            self.player.queue().length(),
        );

        // log::debug!("ui released currently_playing");
    }
}
