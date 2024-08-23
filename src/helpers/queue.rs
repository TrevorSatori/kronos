use std::time::Duration;
use std::{collections::VecDeque, path::PathBuf};
use std::sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex, MutexGuard};

use super::gen_funcs::{path_list_to_song_list, path_to_song, path_to_song_list, Song};

pub struct Queue {
    items: Arc<Mutex<VecDeque<Song>>>,
    selected_item_index: Arc<Mutex<Option<usize>>>,
    total_time: Arc<Mutex<Duration>>,
    tx: Arc<Mutex<Sender<()>>>,
    rx: Arc<Mutex<Receiver<()>>>,
}

fn song_list_to_duration(items: &VecDeque<Song>) -> Duration {
    items.iter().map(|s| s.length).sum()
}

impl Queue {
    pub fn new(queue: Vec<String>) -> Self {
        let (songs, errors) = path_list_to_song_list(queue.iter().map(PathBuf::from).collect());

        if !errors.is_empty() {
            eprintln!("Failed to load some songs.\n{:?}", errors);
        }

        let total_time = song_list_to_duration(&songs);

        let (tx, rx) = channel();

        Self {
            items: Arc::new(Mutex::new(songs)),
            selected_item_index: Arc::new(Mutex::new(None)),
            total_time: Arc::new(Mutex::new(total_time)),
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn songs(&self) -> MutexGuard<VecDeque<Song>> {
        self.items.lock().unwrap()
    }

    pub fn length(&self) -> usize {
        self.songs().len()
    }

    pub fn is_empty(&self) -> bool {
        self.songs().is_empty()
    }

    pub fn paths(&self) -> VecDeque<PathBuf> {
        self.songs().iter().map(|i| i.path.clone()).collect()
    }

    pub fn total_time(&self) -> Duration {
        self.total_time.lock().unwrap().clone()
    }

    pub fn selected_song_index(&self) -> Option<usize> {
        if self.songs().is_empty() {
            None
        } else {
            self.selected_item_index.clone().lock().unwrap().clone()
        }
    }

    pub fn selected_song(&self) -> Option<Song> {
        let songs = self.items.lock().unwrap();
        self.selected_song_index().map(|i| songs[i].clone())
    }

    fn refresh_total_time(&self) {
        let songs = self.items.lock().unwrap();
        *self.total_time.lock().unwrap() = song_list_to_duration(&songs);
    }

    /// Retrieves the first item of the queue, removing it in the process.
    /// This function will block if there is no item available, until there is one.
    pub fn pop(&self) -> Song {
        let rx = self.rx.clone();
        loop {
            let mut items = self.items.lock().unwrap();
            let item = items.pop_front();
            drop(items);
            if let Some(l) = item {
                self.refresh_total_time();
                return l
            }
            rx.lock().unwrap().recv().unwrap_or_else(|e| {
               eprintln!("queue.pop() tried to recv and failed.");
            });
        }
    }

    pub fn select_next(&self) {
        if self.songs().is_empty() {
            return;
        };

        let mut selected_item_index = self.selected_item_index.lock().unwrap();
        *selected_item_index = match *selected_item_index {
            Some(i) => Some(std::cmp::min(i + 1, self.songs().len() - 1)),
            None => Some(0),
        }
    }

    pub fn select_previous(&self) {
        if self.songs().is_empty() {
            return;
        };

        let mut selected_item_index = self.selected_item_index.lock().unwrap();
        *selected_item_index = match *selected_item_index {
            Some(i) => Some(if i > 0 { i - 1 } else { 0 }),
            None => Some(0),
        }
    }

    pub fn select_none(&self) {
        let mut selected_item_index = self.selected_item_index.lock().unwrap();
        *selected_item_index = None;
    }

    pub fn add(&self, path: PathBuf) {
        if path.is_dir() {
            let files = path_to_song_list(&path);
            self.songs().append(&mut VecDeque::from(files));
        } else {
            match path_to_song(&path) {
                Ok(song) => self.songs().push_back(song),
                Err(err) => {
                    eprintln!("Could not add {:?}. Error was {:?}", &path, err);
                }
            }
        }
        self.refresh_total_time();
        self.tx.clone().lock().unwrap().send(()).unwrap();
    }

    pub fn add_front(&self, song: Song) {
        self.songs().push_front(song);
        self.refresh_total_time();
    }

    pub fn remove_selected(&self) {
        if self.songs().is_empty() {
            return;
        }

        let selected_item_index_clone = self.selected_item_index.clone();
        let mut selected_item_index_option = selected_item_index_clone.lock().unwrap();

        let items_clone = self.items.clone();
        let mut items = items_clone.lock().unwrap();

        if let Some(selected_item_index) = *selected_item_index_option {
            {
                items.remove(selected_item_index);

                *selected_item_index_option = if items.is_empty() {
                    None
                } else {
                    Some(selected_item_index.min(items.len() - 1))
                }
            }

            self.refresh_total_time();
        }
    }
}
