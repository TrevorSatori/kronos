use std::sync::{
    Arc, Condvar, Mutex, MutexGuard,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
use std::time::Duration;
use std::collections::VecDeque;

use crate::structs::Song;

pub struct Queue {
    songs: Arc<Mutex<VecDeque<Song>>>,
    selected_item_index: AtomicUsize,

    pop_condvar: Condvar,
    must_exit_pop_loop: AtomicBool,

    queue_length: AtomicUsize,
    total_time: AtomicU64,
}

fn song_list_to_duration(items: &VecDeque<Song>) -> Duration {
    items.iter().map(|s| s.length).sum()
}

impl Queue {
    pub fn new(songs: Vec<Song>) -> Self {
        let songs = VecDeque::from(songs);
        let queue_length = AtomicUsize::new(songs.len());
        let total_time = song_list_to_duration(&songs);

        Self {
            songs: Arc::new(Mutex::new(songs)),
            selected_item_index: AtomicUsize::new(0),

            pop_condvar: Condvar::new(),
            must_exit_pop_loop: AtomicBool::new(false),

            queue_length,
            total_time: AtomicU64::new(total_time.as_secs()),
        }
    }

    /// Retrieves the first item of the queue, removing it in the process.
    /// This function will block if there is no item available, until there is one.
    pub fn pop(&self) -> Result<Song, ()> {
        let target = "::queue.pop()";

        let mut items = self.songs();

        loop {
            if self.must_exit_pop_loop.load(Ordering::SeqCst) {
                log::trace!(target: target, "Exit");
                return Err(());
            }

            if let Some(song) = items.pop_front() {
                log::trace!(target: target, "Got song {:?}", song.title);
                self.queue_length.fetch_sub(1, Ordering::SeqCst);
                self.set_total_time(song_list_to_duration(&items).as_secs());
                return Ok(song);
            }

            log::trace!(target: target, "Waiting for queue change...");
            items = self.pop_condvar.wait(items).unwrap();
        }
    }

    pub fn quit(&self) {
        log::trace!("Queue.quit()");
        self.must_exit_pop_loop.store(true, Ordering::SeqCst);
        self.pop_condvar.notify_one();
    }

    fn mut_queue(&self, f: impl FnOnce(&mut VecDeque<Song>)) {
        let mut songs = self.songs();

        f(&mut *songs);

        self.queue_length.store(songs.len(), Ordering::SeqCst);
        self.set_total_time(song_list_to_duration(&songs).as_secs());

        self.pop_condvar.notify_one();
    }

    pub fn songs(&self) -> MutexGuard<VecDeque<Song>> {
        self.songs.lock().unwrap()
    }

    pub fn length(&self) -> usize {
        self.queue_length.load(Ordering::SeqCst)
    }

    pub fn total_time(&self) -> Duration {
        Duration::new(self.total_time.load(Ordering::SeqCst), 0)
    }

    fn set_total_time(&self, seconds: u64) {
        self.total_time.store(seconds, Ordering::SeqCst);
    }

    pub fn selected_song_index(&self) -> usize {
        self.selected_item_index.load(Ordering::SeqCst)
    }

    pub fn selected_song(&self) -> Option<Song> {
        let songs = self.songs();
        songs.get(self.selected_song_index()).map(|s| s.clone())
    }

    pub fn select_next(&self) {
        let length = self.length();

        if length == 0 {
            return;
        };

        self.selected_item_index.fetch_add(1, Ordering::SeqCst);
        self.selected_item_index.fetch_min(length.saturating_sub(1), Ordering::SeqCst);
    }

    pub fn select_previous(&self) {
        let length = self.length();

        if length == 0 {
            return;
        };

        self.selected_item_index.fetch_sub(1, Ordering::SeqCst);
        self.selected_item_index.fetch_min(length.saturating_sub(1), Ordering::SeqCst);
    }

    pub fn add_front(&self, song: Song) {
        self.mut_queue(|queue_songs| {
            queue_songs.push_front(song);
        });
    }

    pub fn add_back(&self, song: Song) {
        self.mut_queue(|queue_songs| {
            queue_songs.push_back(song);
        });
    }

    pub fn append(&self, songs: &mut VecDeque<Song>) {
        self.mut_queue(|queue_songs| {
            queue_songs.append(songs);
        });
    }

    pub fn remove_selected(&self) {
        if self.length() == 0 {
            return;
        }

        let selected_index = self.selected_song_index();

        self.mut_queue(|queue_songs| {
            queue_songs.remove(selected_index);
        });

        self.select_previous();
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        log::trace!("Player.Queue drop");
    }
}
