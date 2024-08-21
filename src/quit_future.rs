use std::future::Future;
use std::sync::{Arc, Mutex};
use std::task;

pub struct QuitState {
    completed: bool,
    waker: Option<task::Waker>,
}

impl QuitState {
    pub fn complete(&mut self) {
        self.completed = true;

        if let Some(waker) = self.waker.take() {
            waker.wake();
        } else {
            eprintln!("Waker is gone! This is an error");
        }

    }
}

pub struct Quit {
    quit_state: Arc<Mutex<QuitState>>,
}

impl Quit {
    pub fn new() -> Self {
        Self {
            quit_state: Arc::new(Mutex::new(QuitState {
                completed: false,
                waker: None,
            })),
        }
    }

    pub fn state(&self) -> Arc<Mutex<QuitState>> {
        self.quit_state.clone()
    }
}

impl Future for Quit {
    type Output = ();
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let mut shared_state = self.quit_state.lock().unwrap();
        if shared_state.completed {
            task::Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            task::Poll::Pending
        }
    }
}