use crate::app::{App, Browser};
use crate::config::Config;
use crate::helpers::gen_funcs;
use ratatui::widgets::block::Position;
use ratatui::widgets::ListState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};
use crate::helpers::queue::Queue;

impl Browser {
    pub fn top_bar<'a>(self: &Self, cfg: &Config) -> Block<'a> {
        let folder_name = self
            .current_directory
            .file_name()
            .map(|s| s.to_str())
            .flatten()
            .map(String::from)
            .unwrap_or("".to_string());

        let browser_title = match &self.filter {
            Some(filter) => Line::from(vec![
                Span::styled("  search: ", Style::default()),
                Span::styled(
                    filter.clone(),
                    Style::default().fg(Color::Red),
                ),
            ]),
            _ => Line::from(folder_name),
        };

        let top_bar = Block::default()
            .borders(Borders::NONE)
            .title(browser_title)
            .title_alignment(Alignment::Left)
            .title_position(Position::Top)
            .title_style(Style::new().bg(cfg.background()).fg(cfg.foreground()));

        top_bar
    }

    pub fn render_top_bar(self: &Self, cfg: &Config, frame: &mut Frame, area: Rect) {
        frame.render_widget(self.top_bar(cfg), area);
    }

    fn file_list<'a>(self: &Self, cfg: &Config) -> List<'a> {
        let browser_items: Vec<ListItem> = self
            .items
            .items()
            .iter()
            .map(|i| {
                let fg = match self.filter.as_ref() {
                    Some(s) if (i.to_lowercase().contains(&s.to_lowercase())) => Color::Red,
                    _ => Color::Reset,
                };
                ListItem::new(Text::from(i.to_owned())).style(Style::default().fg(fg))
            })
            .collect();

        let browser_list = List::new(browser_items)
            .style(Style::default().fg(cfg.foreground()))
            .highlight_style(
                Style::default()
                    .bg(cfg.highlight_background())
                    .fg(cfg.highlight_foreground())
                    .add_modifier(Modifier::BOLD),
            )
            .scroll_padding(0)
            .highlight_symbol("");

        browser_list
    }

    pub fn render_file_list(self: &Self, cfg: &Config, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            self.file_list(cfg),
            area,
            &mut self.items.state(),
        );
    }
}

fn queue_list<'a>(queue_items: &Queue, cfg: &Config) -> List<'a> {
    let queue_items: Vec<String> = queue_items
        .songs()
        .iter()
        .map(gen_funcs::song_to_string)
        .collect();

    let queue_list = List::new(queue_items)
        .style(Style::default().fg(cfg.foreground()))
        .highlight_style(
            Style::default()
                .bg(cfg.highlight_background())
                .fg(cfg.highlight_foreground())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");

    queue_list
}

pub fn music_tab(frame: &mut Frame, app: &mut App, chunks: Rect, cfg: &Config) {
    let [area_top, area_main] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
        .horizontal_margin(2)
        .split(chunks)
    else {
        panic!("Layout.split() failed");
    };

    let [area_main_left, area_main_separator, area_main_right] = *Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(5),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(area_main)
    else {
        panic!("Layout.split() failed");
    };

    let [_separator_left, separator_middle, _separator_right] = *Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(area_main_separator)
    else {
        panic!("Layout.split() failed");
    };

    app.browser.items.height = area_main_left.height;

    app.browser.render_top_bar(cfg, frame, area_top);
    app.browser.render_file_list(cfg, frame, area_main_left);

    let vertical_separator = Block::default()
        .borders(Borders::RIGHT)
        .border_type(BorderType::Double);
    frame.render_widget(vertical_separator, separator_middle);

    frame.render_stateful_widget(
        queue_list(&app.queue_items, cfg),
        area_main_right,
        &mut ListState::default().with_selected(app.queue_items.selected_song_index()),
    );
}
