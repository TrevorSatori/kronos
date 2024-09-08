use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::config::Config;

pub struct HelpTab<'a> {
    config: Config,
    header: Vec<&'a str>,
    items: Vec<Vec<&'a str>>,
    state: TableState,
}

impl<'a> HelpTab<'a> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            header: vec!["Keys", "Commands"],
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
            state: TableState::default(),
        }
    }

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

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(1)
            .constraints([Constraint::Percentage(50)].as_ref())
            .split(area);

        let header = self
            .header
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(self.config.theme.highlight_foreground)));

        let header = Row::new(header)
            .style(
                Style::default()
                    .bg(self.config.theme.background)
                    .fg(self.config.theme.foreground),
            )
            .height(1)
            .bottom_margin(0);

        let rows = self.items.iter().map(|item| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().map(|c| Cell::from(*c));
            Row::new(cells).height(height as u16).bottom_margin(0)
        });

        let widths = [Constraint::Length(5), Constraint::Length(10)];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .title(" Controls ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Plain),
            )
            .style(
                Style::default()
                    .fg(self.config.theme.foreground)
                    .bg(self.config.theme.background),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(self.config.theme.highlight_background)
                    .fg(self.config.theme.highlight_foreground),
            )
            .widths(&[Constraint::Percentage(50), Constraint::Length(30), Constraint::Min(10)]);
        f.render_stateful_widget(&table, layout[0], &mut self.state);
    }
}
