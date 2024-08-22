use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
    Frame,
};

use crate::config::Config;
use crate::helpers::stateful_table::StatefulTable;

pub fn instructions_tab<'a>(f: &mut Frame, control_table: &mut StatefulTable<'a>, area: Rect, cfg: &Config) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(1)
        .constraints([Constraint::Percentage(50)].as_ref())
        .split(area);

    let header = control_table
        .header
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(cfg.highlight_foreground())));

    let header = Row::new(header)
        .style(Style::default().bg(cfg.background()).fg(cfg.foreground()))
        .height(1)
        .bottom_margin(0);

    let rows = control_table.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    let widths = [Constraint::Length(5), Constraint::Length(10)];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(" Controls ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Plain),
        )
        .style(Style::default().fg(cfg.foreground()).bg(cfg.background()))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(cfg.highlight_background())
                .fg(cfg.highlight_foreground()),
        )
        // .highlight_symbol(">> ")
        .widths(&[Constraint::Percentage(50), Constraint::Length(30), Constraint::Min(10)]);
    f.render_stateful_widget(table, layout[0], &mut control_table.state);
}
