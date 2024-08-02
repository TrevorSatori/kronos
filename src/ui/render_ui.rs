use std::time::Duration;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span},
    widgets::{Block, Borders, Tabs},
    Frame,
    prelude::*,
};
use ratatui::widgets::{BorderType, Gauge};
use crate::app::{App, AppTab};
use crate::config::Config;
use crate::ui::{music_tab, instructions_tab};

pub fn render_ui(f: &mut Frame, app: &mut App, cfg: &Config) {
    let size = f.size();

    let main_layouts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2), Constraint::Length(1)].as_ref())
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

    fn duration_to_string(duration: Duration) -> String {
        seconds_to_string(duration.as_secs())
    }

    fn seconds_to_string(duration: u64) -> String {
        let seconds = duration % 60;
        let minutes = duration.saturating_div(60);
        format!("{:0>2}:{:0>2}", minutes, seconds)
    }

    let playing_file = Block::default()
        .style(Style::default().fg(cfg.foreground()))
        .title(app.current_song())
        .borders(Borders::NONE)
        .title_alignment(Alignment::Center)
        .title_position(ratatui::widgets::block::Position::Bottom);
    f.render_widget(playing_file, main_layouts[2]);

    let playing_gauge_label = format!(
        "{time_played} / {current_song_length}",
        time_played = duration_to_string(app.music_handle.time_played()),
        current_song_length = seconds_to_string(app.music_handle.song_length() as u64),
    );

    let playing_gauge = Gauge::default()
        .style(Style::default().fg(cfg.foreground()))
        .label(playing_gauge_label)
        .gauge_style(Style::default().fg(cfg.highlight_background()))
        .ratio(app.song_progress());
    f.render_widget(playing_gauge, main_layouts[3]);

}
