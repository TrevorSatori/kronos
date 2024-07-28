use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Text},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem},
    Frame,
};
use ratatui::style::Color;
use crate::app::{App, InputMode};
use crate::config::Config;
use crate::helpers::gen_funcs;

pub fn music_tab(frame: &mut Frame, app: &mut App, chunks: Rect, cfg: &Config) {
    let browser_queue = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(chunks);

    let queue_playing = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(100 - cfg.progress_bar()),
                Constraint::Percentage(cfg.progress_bar()),
            ]
            .as_ref(),
        )
        .split(browser_queue[1]);

    let browser_items: Vec<ListItem> = app
        .browser_items
        .items()
        .iter()
        .map(|i| {
            let style = match app.browser_filter.clone()  {
                Some(s) if (i.contains(&s)) => {
                    Style::default().fg(Color::Red)
                },
                _ => Style::default(),
            };
            ListItem::new(Text::from(i.to_owned())).style(style)
        })
        .collect();

    let title: String = match app.input_mode() {
        InputMode::BrowserFilter => "Browser | ".to_owned() + app.browser_filter.clone().unwrap_or("".to_string()).as_str(),
        _ => "Browser".to_string(),
    };

    let browser_list = List::new(browser_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_alignment(Alignment::Left)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(cfg.foreground()))
        .highlight_style(
            Style::default()
                .bg(cfg.highlight_background())
                .fg(cfg.highlight_foreground())
                .add_modifier(Modifier::BOLD),
        )
        .scroll_padding(8)
        .highlight_symbol("");

    frame.render_stateful_widget(browser_list, browser_queue[0], &mut app.browser_items.state());

    let queue_items: Vec<ListItem> = app
        .queue_items
        .items()
        .iter()
        .map(|i| ListItem::new(Text::from(gen_funcs::audio_display(i))))
        .collect();

    let queue_title = format!(
        "| Queue: {queue_items} Songs |{total_time}",
        queue_items = app.queue_items.length(),
        total_time = app.queue_items.total_time(),
    );

    let queue_list = List::new(queue_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(queue_title)
                .title_alignment(Alignment::Left)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(cfg.foreground()))
        .highlight_style(
            Style::default()
                .bg(cfg.highlight_background())
                .fg(cfg.highlight_foreground())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");
    frame.render_stateful_widget(queue_list, queue_playing[0], &mut app.queue_items.state());

    let playing_title = format!("| {current_song} |", current_song = app.current_song());

    let playing = Gauge::default()
        .block(
            Block::default()
                .title(playing_title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(cfg.foreground()))
        .gauge_style(Style::default().fg(cfg.highlight_background()))
        .ratio(app.song_progress());
    frame.render_widget(playing, queue_playing[1]);
}
