use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span},
    widgets::{Block, Borders, Tabs},
    Frame,
    prelude::*,
};

use crate::app::{App, AppTab};
use crate::config::Config;
use crate::ui::{music_tab, instructions_tab};

pub fn render_ui(f: &mut Frame, app: &mut App, cfg: &Config) {
    let size = f.size();

    // chunking from top to bottom, 3 gets tabs displayed, the rest goes to item layouts
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    // Main Background block, covers entire screen
    let block = Block::default().style(Style::default().bg(cfg.background()));
    f.render_widget(block, size);

    // Tab Title items collected
    let titles: Vec<Line> = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(first, Style::default().fg(cfg.highlight_background())), // CHANGE FOR CUSTOMIZATION
                Span::styled(rest, Style::default().fg(cfg.highlight_background())), // These are tab highlights, first vs rest diff colors
            ])
        })
        .collect();

    // Box Around Tab Items
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.active_tab as usize)
        .style(Style::default().fg(cfg.foreground()))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(cfg.background()),
        );
    f.render_widget(tabs, chunks[0]);

    match app.active_tab {
        AppTab::Music => music_tab(f, app, chunks[1], cfg),
        AppTab::Controls => instructions_tab(f, app, chunks[1], cfg),
    };
}
