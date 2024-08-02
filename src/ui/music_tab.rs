use std::time::Duration;
use ratatui::{layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Modifier, Style}, text::{Text}, widgets::{Block, BorderType, Borders, Gauge, List, ListItem}, Frame};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::block::Position;
use ratatui::widgets::{Padding};
use crate::app::{App, InputMode};
use crate::config::Config;
use crate::helpers::gen_funcs;

pub fn music_tab(frame: &mut Frame, app: &mut App, chunks: Rect, cfg: &Config) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks);

    let browser_items: Vec<ListItem> = app
        .browser_items
        .items()
        .iter()
        .map(|i| {
            let fg = match app.browser_filter.as_ref()  {
                Some(s) if (i.to_lowercase().contains(&s.to_lowercase())) => Color::Red,
                _ => Color::Reset,
            };
            ListItem::new(Text::from(i.to_owned())).style(Style::default().fg(fg))
        })
        .collect();

    let folder_name = app.last_visited_path.file_name().map(|s| s.to_str()).flatten().map(String::from).unwrap_or("".to_string());

    let browser_title = match app.input_mode() {
        InputMode::BrowserFilter =>
            Line::from(vec![
                Span::styled(
                    "  search: ",
                    Style::default()
                ),
                Span::styled(
                        app.browser_filter.clone().unwrap_or("".to_string()),
                        Style::default().fg(Color::Red)
                ),
            ]),
        _ => Line::from(format!(" {}", folder_name)),
    };

    let browser_block = Block::default()
        .borders(Borders::RIGHT)
        .title(browser_title)
        .title_alignment(Alignment::Left)
        .title_position(Position::Top)
        .title_style(Style::new()
            .bg(cfg.background())
            .fg(cfg.foreground())
            .add_modifier(Modifier::HIDDEN))
        .border_type(BorderType::Double)
        .padding(Padding::new(1, 1, 1, 2));

    app.browser_items.height = browser_block.inner(main_layout[0]).height;

    let browser_list = List::new(browser_items)
        .block(browser_block)
        .style(Style::default().fg(cfg.foreground()))
        .highlight_style(
            Style::default()
                .bg(cfg.highlight_background())
                .fg(cfg.highlight_foreground())
                .add_modifier(Modifier::BOLD),
        )
        .scroll_padding(0)
        .highlight_symbol("");


    frame.render_stateful_widget(browser_list, main_layout[0], &mut app.browser_items.state());

    let queue_items: Vec<ListItem> = app
        .queue_items
        .items()
        .iter()
        .map(|i| ListItem::new(Text::from(gen_funcs::audio_display(i))))
        .collect();

    let queue_title = format!(
        "queue: {queue_items} songs, {total_time}",
        queue_items = app.queue_items.length(),
        total_time = app.queue_items.total_time(),
    );

    let queue_list = List::new(queue_items)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .title(queue_title)
                .title_alignment(Alignment::Center)
                .padding(Padding::new(1, 1, 1, 1))
        )
        .style(Style::default().fg(cfg.foreground()))
        .highlight_style(
            Style::default()
                .bg(cfg.highlight_background())
                .fg(cfg.highlight_foreground())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");
    frame.render_stateful_widget(queue_list, main_layout[1], &mut app.queue_items.state());
}
