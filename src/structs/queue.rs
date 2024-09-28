use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex, MutexGuard,
};
use std::time::Duration;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::structs::Song;

pub struct Queue {
    items: Arc<Mutex<VecDeque<Song>>>,
    selected_item_index: Arc<Mutex<Option<usize>>>,
    total_time: Arc<Mutex<Duration>>,
    tx: Arc<Mutex<Sender<()>>>,
    rx: Arc<Mutex<Receiver<()>>>,
    must_exit_pop_loop: AtomicBool, // TODO: enum in tx/rx
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
            must_exit_pop_loop: AtomicBool::new(false),
        }
    }

    pub fn quit(&self) {
        log::trace!("Queue.quit()");
        self.must_exit_pop_loop.store(true, Ordering::SeqCst);
        if let Err(err) = self.tx.lock().unwrap().send(()) {
            log::warn!("Queue.quit().send(Stop) failed {:?}", err);
        }
    }

    pub fn notify_queue_change(&self) {
        self.tx.lock().unwrap().send(()).unwrap();
    }

    pub fn songs(&self) -> MutexGuard<VecDeque<Song>> {
        self.items.lock().unwrap()
    }

    pub fn length(&self) -> usize {
        self.songs().len()
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
    pub fn pop(&self) -> Result<Song, ()> {
        loop {
            let song = {
                let mut items = self.items.lock().unwrap();
                items.pop_front()
            };
            if let Some(song) = song {
                self.refresh_total_time();
                return Ok(song);
            }
            if let Err(err) = self.rx.lock().unwrap().recv() {
                log::warn!("Queue.pop() {:#?}", err);
                break;
            }
            if self.must_exit_pop_loop.load(Ordering::SeqCst) {
                log::debug!("Queue.pop() exit");
                break;
            }
            log::trace!("Queue.pop() iteration");
        }

        Err(())
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

impl Drop for Queue {
    fn drop(&mut self) {
        log::trace!("Player.Queue drop");
    }
}
