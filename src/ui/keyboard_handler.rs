use crossterm::event::KeyEvent;

pub trait KeyboardHandler {
    fn on_key(&self, key: KeyEvent) -> bool { false }
}
