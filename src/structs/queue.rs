use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex, MutexGuard,
};
use std::time::Duration;
use std::collections::VecDeque;

use crate::structs::Song;

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
    pub fn new(songs: Vec<Song>) -> Self {
        let songs = VecDeque::from(songs);
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

    fn notify_queue_change(&self) {
        self.tx.lock().unwrap().send(()).unwrap();
    }

    pub fn songs(&self) -> MutexGuard<VecDeque<Song>> {
        self.items.lock().unwrap()
    }

    pub fn length(&self) -> usize {
        self.songs().len()
    }

    pub fn is_empty(&self) -> bool {
        self.length() == 0
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
        loop {
            let item = {
                let mut items = self.items.lock().unwrap();
                items.pop_front()
            };
            if let Some(l) = item {
                self.refresh_total_time();
                return l;
            }
            self.rx.lock().unwrap().recv().unwrap_or_else(|e| {
                log::error!("queue.pop() - no more messages. {:#?}", e);
                // TODO(BUG):
                //   If we're here, then rx.lock will keep returning an error,
                //   so this loop will start spinning.
                //   Yet we own the channel sender, so, if we're here, we're being dropped.
                //   Does the drop exit the loop automagically?
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

    pub fn add_front(&self, song: Song) {
        self.songs().push_front(song);
        self.refresh_total_time();
        self.notify_queue_change();
    }

    pub fn add_back(&self, song: Song) {
        self.songs().push_back(song);
        self.refresh_total_time();
        self.notify_queue_change();
    }

    pub fn append(&self, songs: &mut VecDeque<Song>) {
        self.songs().append(songs);
        self.refresh_total_time();
        self.notify_queue_change();
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

            drop(items);
            self.refresh_total_time();
        }
    }
}
