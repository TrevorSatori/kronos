use std::path::PathBuf;
use ratatui::widgets::ListState;

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    curr: usize,
    pub offset: usize,
    pub height: u16,
    pub padding: u16,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
            curr: 0,
            offset: 0,
            height: 0,
            padding: 6,
        }
    }

    pub fn selected_index(&self) -> usize {
        self.curr
    }

    // return all items contained in vector
    pub fn items(&self) -> &Vec<T> {
        &self.items
    }

    // return item at index
    pub fn item(&self) -> &T {
        &self.items[self.curr]
    }

    pub fn state(&self) -> ListState {
        self.state.clone().with_offset(self.offset)
    }

    pub fn empty(&self) -> bool {
        self.items.is_empty()
    }

    fn padding_top(&self) -> usize {
        6
    }

    fn padding_bottom(&self) -> usize {
        usize::from(self.height.saturating_sub(self.padding))
    }

    pub fn set_offset(&mut self, i: usize, padding: usize) {
        self.offset = if i > padding {
            (i - padding).min(self.items.len() - usize::from(self.height))
        } else {
            0
        };
    }

    pub fn next(&mut self) {
        self.next_by(1)
    }

    pub fn next_by(&mut self, amount: usize) {
        if self.items.is_empty() {
            return;
        };

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    self.items.len() - 1
                } else {
                    i + amount
                }
            }
            None => 0,
        };
        self.curr = i;
        self.state.select(Some(i));

        if i > self.offset + self.padding_bottom() {
            self.set_offset(i, self.padding_bottom());
        }
    }

    pub fn previous(&mut self) {
        self.previous_by(1)
    }

    pub fn previous_by(&mut self, amount: usize) {
        if self.items.is_empty() {
            return;
        };

        let i = match self.state.selected() {
            Some(i) if i > amount => {
                i - amount
            }
            _ => 0,
        };
        self.curr = i;
        self.state.select(Some(i));

        if i < self.offset + self.padding_top() {
            self.set_offset(i, self.padding_top());
        }
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
    pub fn select(&mut self, i: usize) {
        self.curr = i;
        self.state.select(Some(i));
    }
}

impl<T: ToString> StatefulList<T> {
    pub fn find_by_path(&self, s: &PathBuf) -> usize {
        let mut i = 0;

        for n in 0 .. self.items.len() {
            if s.ends_with(self.items[n].to_string()) {
                i = n;
                break;
            }
        }

        i
    }

    pub fn next_index_wrapped(&self, i: usize) -> usize {
        if i >= self.items.len() - 1 {
            0
        } else {
            i + 1
        }
    }

    pub fn previous_index_wrapped(&self, i: usize) -> usize {
        if i == 0 {
            self.items.len() - 1
        } else {
            i - 1
        }
    }

    pub fn find_next_by_match(&self, s: &str, direction_forward: bool) -> Option<usize> {
        let mut i: usize = self.curr;

        loop {
            i = if direction_forward { self.next_index_wrapped(i) } else { self.previous_index_wrapped(i) };

            if i == self.curr {
                return None;
            }

            if self.items[i].to_string().to_lowercase().contains(&s.to_lowercase()) {
                return Some(i);
            }
        }
    }

    pub fn select_by_path(&mut self, s: &PathBuf) {
        self.select(self.find_by_path(s));
    }

    pub fn select_next_by_match(&mut self, s: &str) {
        if let Some(i) = self.find_next_by_match(s, true) {
            self.select(i);
        }
    }

    pub fn select_previous_by_match(&mut self, s: &str) {
        if let Some(i) = self.find_next_by_match(s, false) {
            self.select(i);
        }
    }
}
