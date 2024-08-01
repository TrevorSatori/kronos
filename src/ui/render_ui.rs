use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span},
    widgets::{Block, Borders, Tabs},
    Frame,
    prelude::*,
};
use ratatui::widgets::{BorderType, Padding};
use crate::app::{App, AppTab};
use crate::config::Config;
use crate::ui::{music_tab, instructions_tab};

pub fn render_ui(f: &mut Frame, app: &mut App, cfg: &Config) {
    let size = f.size();

    let main_layouts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let block = Block::default().style(Style::default().bg(cfg.background()));
    f.render_widget(block, size);

    let titles: Vec<Line> = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(first, Style::default().fg(cfg.foreground())),
                Span::styled(rest, Style::default().fg(cfg.foreground())),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::BOTTOM | Borders::TOP)
            .border_type(BorderType::QuadrantInside)
            .border_style(Style::default().fg(Color::from_hsl(29.0, 34.0, 20.0)).bg(cfg.background()))
        )
        .select(app.active_tab as usize)
        .style(
            Style::default()
                .fg(cfg.foreground())
                .bg(Color::from_hsl(29.0, 34.0, 20.0))
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::from_hsl(39.0, 67.0, 69.0))
                // .bg(cfg.highlight_background()),
        );
    f.render_widget(tabs, main_layouts[0]);

    match app.active_tab {
        AppTab::Music => music_tab(f, app, main_layouts[1], cfg),
        AppTab::Controls => instructions_tab(f, app, main_layouts[1], cfg),
    };
}
