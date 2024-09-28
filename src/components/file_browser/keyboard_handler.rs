use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ui::KeyboardHandlerMut;

use super::FileBrowser;

impl<'a> KeyboardHandlerMut<'a> for FileBrowser<'a> {
    fn on_key(&mut self, key: KeyEvent) -> bool {
        if !self.filter.is_some() {
            self.on_normal_key_event(key);
        } else {
            self.on_filter_key_event(key);
        }

        true
    }
}

impl<'a> FileBrowser<'a> {
    fn on_normal_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Backspace => self.navigate_up(),
            KeyCode::Down => {
                self.select_next();
            },
            KeyCode::Up => {
                self.select_previous();
            },
            // KeyCode::PageUp => self.items.previous_by(5),
            // KeyCode::PageDown => self.items.next_by(5),
            KeyCode::End => self.select_last(),
            KeyCode::Home => self.select_first(),
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                self.filter = Some("".to_string());
            }
            KeyCode::Enter | KeyCode::Char(_) => {
                self.enter_selection(key);
            },
            _ => {}
        }
    }

    fn on_filter_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter if key.modifiers == KeyModifiers::ALT => {
                self.enter_selection(key);
            }
            KeyCode::Enter => {
                self.filter = None;
                self.enter_selection(key);
            }
            KeyCode::Esc => {
                self.filter = None;
            }
            KeyCode::Down => {
                self.select_next_match();
            }
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                self.select_next_match();
            }
            KeyCode::Up => {
                self.select_previous_match();
            }
            KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => {
                self.select_previous_match();
            }
            KeyCode::Backspace => {
                self.filter_delete();
            }
            KeyCode::Char(char) => {
                self.filter_append(char);
            }
            _ => {}
        }
    }
}
