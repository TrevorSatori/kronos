use tui::widgets::ListState;

// TODO encapsulation
pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    curr: usize,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
            curr: 0,
        }
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
        self.state.clone()
    }

    pub fn empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn next(&mut self) {
        // check if empty
        if self.items.is_empty() {
            return;
        };

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.curr = i;
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        // check if empty
        if self.items.is_empty() {
            return;
        };

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.curr = i;
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
