use std::{
    env,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use kronos::gen_funcs;
use kronos::music_handler::MusicHandle;
use kronos::queue::Queue;
use kronos::stateful_list::StatefulList;
use kronos::stateful_table::StatefulTable;
use crate::state::{save_state, State};

#[derive(Clone, Copy)]
pub enum InputMode {
    Browser,
    Queue,
    Controls,
}

/// Represents the active tab state.
#[derive(Debug, Clone, Copy)]
pub enum AppTab {
    Music = 0,
    Controls,
}

impl AppTab {
    /// Get the next tab in the list.
    pub fn next(&self) -> Self {
        match self {
            Self::Music => Self::Controls,
            // Wrap around to the first tab.
            Self::Controls => Self::Music,
        }
    }
}

pub struct App<'a> {
    pub browser_items: StatefulList<String>,
    pub queue_items: Queue,
    pub control_table: StatefulTable<'a>,
    pub music_handle: MusicHandle,
    input_mode: InputMode,
    pub titles: Vec<&'a str>,
    pub active_tab: AppTab,
    pub last_visited_path: PathBuf,
}

impl<'a> App<'a> {
    pub fn new(initial_directory: Option<String>) -> Self {
        if let Some(path) = initial_directory {
            env::set_current_dir(&path).unwrap_or_else(|err| {
                eprintln!("Could not set_current_dir to last_visited_path\n\tPath: {}\n\tError: {:?}", path, err);
            });
        }

        Self {
            browser_items: StatefulList::with_items(gen_funcs::scan_and_filter_directory()),
            queue_items: Queue::with_items(),
            control_table: StatefulTable::new(),
            music_handle: MusicHandle::new(),
            input_mode: InputMode::Browser,
            titles: vec!["Music", "Controls"],
            active_tab: AppTab::Music,
            last_visited_path: env::current_dir().unwrap(),
        }
    }

    pub fn save_state(self) {
        save_state(State {
            last_visited_path: self.last_visited_path.to_str().map(String::from),
        }).unwrap_or_else(|error| {
            eprintln!("Error in save_state {}", error);
        });
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

    pub fn current_song(&self) -> String {
        if self.music_handle.sink_empty() && self.queue_items.is_empty() {
            "CURRENT SONG".to_string()
        } else {
            self.music_handle.currently_playing()
        }
    }

    // if item selected is folder, enter folder, else play record.
    pub fn evaluate(&mut self) {
        let join = self.selected_item();
        // if folder enter, else play song
        if join.is_dir() {
            self.last_visited_path = join.clone();
            env::set_current_dir(join).unwrap();
            self.browser_items = StatefulList::with_items(gen_funcs::scan_and_filter_directory());
            self.browser_items.next();
        } else {
            self.music_handle.play(join);
        }
    }

    // cd into selected directory
    pub fn backpedal(&mut self) {
        env::set_current_dir("../").unwrap();
        self.browser_items = StatefulList::with_items(gen_funcs::scan_and_filter_directory());
        self.browser_items.select_by_path(&self.last_visited_path);
    }

    // if queue has items and nothing playing, auto play
    pub fn auto_play(&mut self) {
        thread::sleep(Duration::from_millis(250));
        if self.music_handle.sink_empty() && !self.queue_items.is_empty() {
            self.music_handle.set_time_played(0);
            self.music_handle.play(self.queue_items.pop());
        }
    }

    // if playing and
    pub fn song_progress(&mut self) -> f64 {
        // edge case if nothing queued or playing
        if self.music_handle.sink_empty() && self.queue_items.is_empty() {
            0.0

        // if something playing, calculate progress
        } else if !self.music_handle.sink_empty() {
            f64::clamp(self.music_handle.time_played() as f64 / self.music_handle.song_length() as f64, 0.0, 1.0)
        // if nothing playing keep rolling
        } else {
            self.auto_play();
            0.0
        }
    }

    // get file path
    pub fn selected_item(&self) -> PathBuf {
        let current_dir = env::current_dir().unwrap();
        if self.browser_items.empty() {
            Path::new(&current_dir).into()
        } else {
            let join = Path::join(&current_dir, Path::new(&self.browser_items.item()));
            join
        }
    }
}
