use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ui::KeyboardHandlerMut;

use super::FileBrowser;

impl<'a> KeyboardHandlerMut<'a> for FileBrowser<'a> {
    fn on_key(&mut self, key: KeyEvent) -> bool {
        if !self.filter.is_some() {
            on_normal_key_event(self, key);
        } else {
            on_filter_key_event(self, key);
        }

        true
    }
}

fn on_normal_key_event(browser: &mut FileBrowser, key: KeyEvent) {
    match key.code {
        // KeyCode::Enter => { browser.enter_selection(key); },
        KeyCode::Backspace => browser.navigate_up(),
        KeyCode::Down => browser.items.next(),
        KeyCode::Up => browser.items.previous(),
        KeyCode::PageUp => browser.items.previous_by(5),
        KeyCode::PageDown => browser.items.next_by(5),
        KeyCode::End => browser.select_last(),
        KeyCode::Home => browser.items.select(0),
        KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
            browser.filter = Some("".to_string());
        }
        KeyCode::Enter | KeyCode::Char(_) => {
            browser.enter_selection(key);
        },
        _ => {}
    }
}

fn on_filter_key_event(browser: &mut FileBrowser, key: KeyEvent) {
    match key.code {
        KeyCode::Enter if key.modifiers == KeyModifiers::ALT => {
            browser.enter_selection(key);
        }
        KeyCode::Enter => {
            browser.filter = None;
            browser.enter_selection(key);
        }
        KeyCode::Esc => {
            browser.filter = None;
        }
        KeyCode::Down => {
            browser.select_next_match();
        }
        KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
            browser.select_next_match();
        }
        KeyCode::Up => {
            browser.select_previous_match();
        }
        KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => {
            browser.select_previous_match();
        }
        KeyCode::Backspace => {
            browser.filter_delete();
        }
        KeyCode::Char(char) => {
            browser.filter_append(char);
        }
        _ => {}
    }
}
