use std::time::Duration;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Gauge, Tabs},
    Frame,
};

use crate::{
    app::AppTab,
    config::Config,
    constants::{MAIN_SECTIONS, SECONDS_PER_HOUR, SECONDS_PER_MINUTE},
    helpers::gen_funcs::{song_to_string, Song},
};

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

pub fn render_top_bar(frame: &mut Frame, config: &Config, area: Rect, active_tab: AppTab) {
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

pub fn render_playing_gauge(
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
