use std::error::Error;
use std::sync::{mpsc::Receiver, Arc, Mutex, MutexGuard};
use std::{env, path::PathBuf, thread, time::Duration};
use std::io::BufRead;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use log::error;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::{Style, Widget},
    widgets::{Block, WidgetRef},
};
use rodio::OutputStream;

use crate::{
    config::Config,
    player::Player,
    state::State,
    term::set_terminal,
    ui,
    ui::{CurrentlyPlaying, KeyboardHandlerEnum, KeyboardHandlerMut, TopBar},
    Command,
    components::{FileBrowser, FileBrowserSelection},
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum FocusedElement {
    Browser,
    Queue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Library = 0,
    Playlists,
    FileBrowser,
    Help,
}

pub struct App<'a> {
    must_quit: bool,
    config: Config,

    _music_output: OutputStream,
    player: Arc<Player>,
    player_command_receiver: Arc<Mutex<Receiver<Command>>>,

    focused_element: FocusedElement,
    target: Option<KeyboardHandlerEnum<'a>>,
    active_tab: AppTab,

    library: Arc<ui::Library<'a>>,
    playlist: Arc<ui::Playlists<'a>>,
    browser: Arc<Mutex<FileBrowser<'a>>>,
    help_tab: Arc<Mutex<ui::HelpTab<'a>>>,
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

        let mut browser = FileBrowser::new(config.theme, current_directory, player.queue());
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
            focused_element: FocusedElement::Browser,
            active_tab: AppTab::Library,
            browser: Arc::new(Mutex::new(browser)),
            help_tab: Arc::new(Mutex::new(ui::HelpTab::new(config))),
            player_command_receiver: Arc::new(Mutex::new(player_command_receiver)),
            player,
            playlist,
            library,
            target: None,
        }
    }

    fn file_browser(&self) -> MutexGuard<FileBrowser<'a>>  {
        self.browser.lock().unwrap()
    }

    fn to_state(&self) -> State {
        let queue_items = self.player.queue().songs().clone();
        let playlists = self.playlist.playlists();

        State {
            last_visited_path: self.file_browser().current_directory().to_str().map(String::from),
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
            terminal.draw(|frame| {
                frame.render_widget_ref(&*self, frame.size());
            })?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.on_key(key);
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
        log::debug!("file_browser().on_select({:?}, {:?})", file_browser_selection, key_event);
        match (file_browser_selection, key_event.code) {
            (FileBrowserSelection::Song(song), KeyCode::Enter) => {
                player.play_song(song);
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Enter) => {
                player.enqueue_cue(cue_sheet);
            }
            (FileBrowserSelection::Song(song), KeyCode::Char('j')) => {
                log::debug!("TODO: file_browser().on_select(Song({}), j)", song.title);
                media_library.add_song(song.clone());
            }
            (FileBrowserSelection::Song(song), KeyCode::Char('a')) => {
                player.enqueue_song(song);
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Char('a')) => {
                player.enqueue_cue(cue_sheet);
            }
            (FileBrowserSelection::Directory(path), KeyCode::Char('a')) => {
                log::debug!("TODO: file_browser().on_select(Directory({}), a)", path.display());
                // directory_to_songs_and_folders
            }
            (FileBrowserSelection::Song(song), KeyCode::Char('y')) => {
                playlists.add_song(song);
            }
            (FileBrowserSelection::CueSheet(cue_sheet), KeyCode::Char('y')) => {
                playlists.add_cue(cue_sheet);
            }
            (FileBrowserSelection::Directory(path), KeyCode::Char('y')) => {
                log::debug!("TODO: file_browser().on_select(Directory({}), y)", path.display());
                // directory_to_songs_and_folders
            }
            (FileBrowserSelection::Directory(path), KeyCode::Char('j')) => {
                log::debug!("TODO: file_browser().on_select(Directory({}), j)", path.display());
                // let songs = path_to
                // media_library.lock().unwrap().push(song.clone());
                // directory_to_songs_and_folders
            }
            _ => {}
        }
    }

    fn spawn_terminal(&self) {
        let cwd = self.file_browser().current_directory().clone();

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

}

impl<'a> KeyboardHandlerMut<'a> for App<'a> {
    fn on_key(&mut self, key: KeyEvent) -> bool {
        let mut handled = true;

        let focus_trapped = self.focused_element == FocusedElement::Browser && self.file_browser().filter().is_some();
        if !focus_trapped {
            match key.code {
                KeyCode::Right => self.player.seek_forward(),
                KeyCode::Left => self.player.seek_backward(),
                KeyCode::Char('-') => self.player.change_volume(-0.05),
                KeyCode::Char('+') => self.player.change_volume(0.05),
                KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => self.player.toggle(),
                KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => self.player.stop(),
                KeyCode::Char('c') if key.modifiers == KeyModifiers::ALT => self.spawn_terminal(),
                KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => {
                    self.must_quit = true;
                }
                KeyCode::Char('1') => {
                    self.active_tab = AppTab::Library;
                    self.target = Some(KeyboardHandlerEnum::Immut(self.library.clone()));
                }
                KeyCode::Char('2') => {
                    self.active_tab = AppTab::Playlists;
                    self.target = Some(KeyboardHandlerEnum::Immut(self.playlist.clone()));
                }
                KeyCode::Char('3') => {
                    self.active_tab = AppTab::FileBrowser;
                    self.target = Some(KeyboardHandlerEnum::Mut(self.browser.clone()));
                }
                KeyCode::Char('4') => {
                    self.active_tab = AppTab::Help;
                    self.target = Some(KeyboardHandlerEnum::Mut(self.help_tab.clone()));
                }
                KeyCode::Tab if self.active_tab == AppTab::FileBrowser && self.file_browser().filter().is_none() => {
                    self.focused_element = match self.focused_element {
                        FocusedElement::Browser => FocusedElement::Queue,
                        _ => FocusedElement::Browser,
                    };

                    // TODO: focus/blur colors
                    match self.focused_element {
                        FocusedElement::Browser => {
                            self.file_browser().focus();
                            self.player.queue().select_none();
                            self.target = Some(KeyboardHandlerEnum::Mut(self.browser.clone()));
                        }
                        FocusedElement::Queue => {
                            self.file_browser().blur();
                            self.player.queue().select_next();
                            self.target = Some(KeyboardHandlerEnum::Immut(self.player.clone()));
                        }
                    };
                }
                _ => {
                    handled = false;
                }
            };
        };

        if focus_trapped || !handled {
            if let Some(target) = &self.target {
                match target {
                    KeyboardHandlerEnum::Immut(target) => {
                        target.on_key(key);
                    }
                    KeyboardHandlerEnum::Mut(target) => {
                        target.lock().unwrap().on_key(key);
                    }
                }
            }
        }

        true
    }
}

impl<'a> WidgetRef for &App<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().style(Style::default().bg(self.config.theme.background));
        block.render(area, buf);

        let [area_top, _, area_center, area_bottom] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0), Constraint::Length(3)]).areas(area);

        let top_bar = TopBar::new(self.config.theme, self.active_tab);
        top_bar.render(area_top, buf);

        match self.active_tab {
            AppTab::Library => {
                self.library.render_ref(area_center, buf);
            },
            AppTab::Playlists => {
                self.playlist.render_ref(area_center, buf);
            },
            AppTab::FileBrowser => {
                let file_browser = &*self.file_browser();
                file_browser.render_ref(area_center, buf);
            },
            AppTab::Help => {
                self.help_tab.lock().unwrap().render_ref(area_center, buf);
            },
        };

        let currently_playing = CurrentlyPlaying::new(
            self.config.theme,
            self.player.currently_playing().lock().unwrap().clone(),
            self.player.get_pos(),
            self.player.queue().total_time(),
            self.player.queue().length(),
        );
        currently_playing.render(area_bottom, buf);
    }
}
