use ratatui::widgets::TableState;

pub struct StatefulTable<'a> {
    pub header: Vec<&'a str>,
    pub state: TableState,
    pub items: Vec<Vec<&'a str>>,
}

impl<'a> Default for StatefulTable<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulTable<'a> {
    pub fn new() -> Self {
        Self {
            header: vec!["Keys", "Commands"],
            state: TableState::default(),
            items: vec![
                vec!["Q", "Quit"],
                vec!["P", "Play / Pause"],
                vec!["G", "Skip Song"],
                vec!["A", "Add To Queue"],
                vec!["R", "Remove From Queue"],
                vec!["Enter", "Enter Directory"],
                vec!["Backspace", "Previous Directory"],
                vec!["Down", "Next Item"],
                vec!["Up", "Previous Item"],
                vec!["Right / Left", "Enter Queue / Browser"],
                vec!["Tab", "Change Tabs"],
                vec!["+", "Volume Up"],
                vec!["-", "Volume Down"],
            ],
        }
    }

    // pub fn header(&self) -> Vec<&'a str> {
    //     self.header
    // }

    // pub fn items(&self) -> Vec<Vec<&'a str>> {
    //     self.items
    // }

    pub fn next(&mut self) {
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
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
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
        self.state.select(Some(i));
    }
}
