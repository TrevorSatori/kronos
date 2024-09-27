use std::sync::{Arc, Mutex};
use crossterm::event::KeyEvent;

pub trait KeyboardHandler<'a>: 'a {
    fn on_key(&self, key: KeyEvent) -> bool { false }
}
pub trait KeyboardHandlerMut<'a>: 'a {
    fn on_key(&mut self, key: KeyEvent) -> bool { false }
}

pub enum KeyboardHandlerEnum<'a> {
    Immut(Arc<dyn 'a + KeyboardHandler<'a>>),
    Mut(Arc<Mutex<dyn 'a + KeyboardHandlerMut<'a>>>),
}
