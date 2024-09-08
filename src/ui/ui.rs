use log::error;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Gauge, Tabs},
    Frame,
};
use std::time::Duration;

use crate::{
    app::AppTab,
    config::Config,
    constants::{MAIN_SECTIONS, SECONDS_PER_HOUR, SECONDS_PER_MINUTE},
    structs::song::Song,
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

pub fn song_to_string(song: &Song) -> String {
    let title = song.title.clone();

    if let Some(artist) = &song.artist {
        format!("{artist} - {title}")
    } else {
        title
    }
}

pub fn render_top_bar(frame: &mut Frame, config: &Config, area: Rect, active_tab: AppTab) {
    let tab_titles: Vec<Line> = MAIN_SECTIONS
        .iter()
        .map(|t| {
            Line::from(Span::styled(
                t.to_string(),
                Style::default().fg(config.theme.foreground),
            ))
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(config.theme.background).bg(config.theme.background)),
        )
        .select(active_tab as usize)
        .style(
            Style::default()
                .fg(config.theme.foreground)
                .bg(config.theme.top_bar_background),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(config.theme.top_bar_highlight),
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
    // log::debug!("render_playing_gauge");
    let [area_top, area_bottom] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)].as_ref())
        .split(area)
    else {
        panic!("Layout.split() failed");
    };

    // log::debug!("render_playing_gauge current_song {:?}", current_song);

    if let Some(current_song) = current_song {
        let playing_file = Block::default()
            .style(Style::default().fg(config.theme.foreground))
            .title(song_to_string(&current_song))
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center)
            .title_position(ratatui::widgets::block::Position::Bottom);
        frame.render_widget(playing_file, area_top);
    }

    let playing_song_label = current_song.as_ref().map(|song| {
        format!(
            "{time_played} / {current_song_length}",
            time_played = duration_to_string(current_song_position),
            current_song_length = duration_to_string(song.length),
        )
    });

    let songs = if queue_song_count == 1 { "song" } else { "songs" };

    let queue_label = if queue_song_count > 0 {
        Some(format!(
            "{queue_items} {songs} / {total_time} in queue",
            total_time = duration_to_string(queue_total_time),
            queue_items = queue_song_count,
        ))
    } else {
        None
    };

    let playing_gauge_label = match (playing_song_label, queue_label) {
        (Some(playing_song_label), Some(queue_label)) => {
            format!("{playing_song_label}  |  {queue_label}")
        }
        (None, Some(queue_label)) => {
            format!("{queue_label}")
        }
        (Some(playing_song_label), None) => {
            format!("{playing_song_label}")
        }
        _ => "".to_string(),
    };

    if playing_gauge_label.len() > 0 {
        let song_progress = match current_song {
            Some(song) => match song.length.as_secs_f64() {
                0.0 => {
                    error!("Song length is zero! {:?}", song.path);
                    0.0
                }
                n => f64::clamp(current_song_position.as_secs_f64() / n, 0.0, 1.0),
            },
            _ => 0.0,
        };

        let playing_gauge = Gauge::default()
            .style(Style::default().fg(config.theme.foreground))
            .label(playing_gauge_label)
            .gauge_style(Style::default().fg(config.theme.highlight_background))
            .ratio(song_progress);
        frame.render_widget(playing_gauge, area_bottom);
    }
}
