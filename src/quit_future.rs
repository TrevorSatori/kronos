use std::sync::{Arc, Mutex};
use std::task;

pub struct QuitState {
    pub(crate) completed: bool,
    pub(crate) waker: Option<task::Waker>,
}

pub struct Quit {
    pub(crate) shared_state: Arc<Mutex<QuitState>>,
}

impl std::future::Future for Quit {
    type Output = ();
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            task::Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            task::Poll::Pending
        }
    }
}