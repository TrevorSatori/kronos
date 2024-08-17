use crate::app::{App, AppTab};
use crate::config::Config;
use crate::constants::{SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use crate::helpers::gen_funcs::{song_to_string, Song};
use crate::ui::{instructions_tab, music_tab};
use ratatui::widgets::{BorderType, Gauge};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs},
    Frame,
};
use std::time::Duration;

static MAIN_SECTIONS: [&str; 2] = ["Music", "Help"];

fn duration_to_string(total_time: Duration) -> String {
    let hours = total_time.as_secs() / SECONDS_PER_HOUR;
    let minutes = (total_time.as_secs() % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let seconds = total_time.as_secs() % SECONDS_PER_MINUTE;

    let mut time_parts = vec![];

    if hours > 0 {
        time_parts.push(hours);
    }

    time_parts.push(minutes);
    time_parts.push(seconds);

    let strings: Vec<String> = time_parts.iter().map(|s| format!("{:0>2}", s)).collect();
    strings.join(":")
}

pub fn render_ui(frame: &mut Frame, app: &mut App, config: &Config, active_tab: AppTab, current_song: &Option<Song>) {
    let size = frame.size();

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(size);

    let block = Block::default().style(Style::default().bg(config.background()));
    frame.render_widget(block, size);

    render_top_bar(frame, config, areas[0], active_tab);

    match active_tab {
        AppTab::Music => music_tab(frame, app, areas[1], config),
        AppTab::Controls => instructions_tab(frame, app, areas[1], config),
    };

    render_playing_gauge(
        frame,
        config,
        areas[2],
        current_song,
        app.player_sink().get_pos(),
        app.queue_items.total_time(),
        app.queue_items.length(),
    );
}

fn render_top_bar(frame: &mut Frame, config: &Config, area: Rect, active_tab: AppTab) {
    let tab_titles: Vec<Line> = MAIN_SECTIONS
        .iter()
        .map(|t| {
            Line::from(Span::styled(
                t.to_string(),
                Style::default().fg(config.foreground()),
            ))
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(config.background()).bg(config.background())),
        )
        .select(active_tab as usize)
        .style(
            Style::default()
                .fg(config.foreground())
                .bg(Color::from_hsl(29.0, 34.0, 20.0)),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::from_hsl(39.0, 67.0, 69.0)), // .bg(cfg.highlight_background()),
        );
    frame.render_widget(tabs, area);
}

fn render_playing_gauge(
    frame: &mut Frame,
    config: &Config,
    area: Rect,
    current_song: &Option<Song>,
    current_song_position: Duration,
    queue_total_time: Duration,
    queue_song_count: usize,
) {
    let [area_top, area_bottom] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)].as_ref())
        .split(area)
    else {
        panic!("Layout.split() failed");
    };

    if let Some(current_song) = current_song {
        let playing_file = Block::default()
            .style(Style::default().fg(config.foreground()))
            .title(song_to_string(&current_song))
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center)
            .title_position(ratatui::widgets::block::Position::Bottom);
        frame.render_widget(playing_file, area_top);
    }

    let playing_gauge_label_current_song = match current_song {
        Some(song) => format!(
            "{time_played} / {current_song_length}",
            time_played = duration_to_string(current_song_position),
            current_song_length = duration_to_string(song.length),
        ),
        _ => "".to_string(),
    };

    let playing_gauge_label = format!(
        "{playing_gauge_label_current_song} — {total_time}, {queue_items} songs",
        total_time = duration_to_string(queue_total_time),
        queue_items = queue_song_count,
    );

    let song_progress = match current_song {
        Some(song) => f64::clamp(
            current_song_position.as_secs_f64() / song.length.as_secs_f64(),
            0.0,
            1.0,
        ),
        _ => 0.0,
    };

    let playing_gauge = Gauge::default()
        .style(Style::default().fg(config.foreground()))
        .label(playing_gauge_label)
        .gauge_style(Style::default().fg(config.highlight_background()))
        .ratio(song_progress);
    frame.render_widget(playing_gauge, area_bottom);
}
