use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Color},
    text::{Text, Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Padding},
    Frame,
};
use ratatui::widgets::block::Position;

use crate::app::{App, InputMode};
use crate::config::Config;
use crate::helpers::gen_funcs;

fn top_bar<'a>(app: &mut App, cfg: &Config) -> Block<'a> {
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
        _ => Line::from(folder_name),
    };

    let top_bar = Block::default()
        .borders(Borders::NONE)
        .title(browser_title)
        .title_alignment(Alignment::Left)
        .title_position(Position::Top)
        .title_style(Style::new()
            .bg(cfg.background())
            .fg(cfg.foreground())
        );

    top_bar
}

pub fn music_tab(frame: &mut Frame, app: &mut App, chunks: Rect, cfg: &Config) {
    let vertical_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
        .horizontal_margin(2)
        .split(chunks);

    frame.render_widget(top_bar(app, cfg), vertical_areas[0]);

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(vertical_areas[1]);

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

    let browser_block = Block::default()
        .borders(Borders::RIGHT)
        .border_type(BorderType::Double);

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

    let queue_list = List::new(queue_items)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(Padding::left(1))
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
