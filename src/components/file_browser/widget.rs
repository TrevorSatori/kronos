use std::path::Path;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::{Line, Modifier, Span, Style},
    style::Color,
    text::Text,
    widgets::{
        block::Position,
        Block,
        BorderType,
        Borders,
        List,
        WidgetRef,
        StatefulWidget,
    },
};

use crate::{
    structs::{Queue},
    ui,
    ui::stateful_list::StatefulList,
    config::{Theme},
};

use super::FileBrowser;

impl<'a> WidgetRef for &FileBrowser<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let (area_top, area_main_left, area_main_separator, area_main_right) = create_areas(area);

        // self.set_height(area_main_left.height);
        // TODO: not use stateful list

        let tb = top_bar(&self.theme, self.current_directory(), &self.filter);
        tb.render_ref(area_top, buf);

        let fl = file_list(&self.theme, self.items(), &self.filter());
        StatefulWidget::render(
            fl,
            area_main_left,
            buf,
            &mut self.items.state(),
        );

        let [_separator_left, separator_middle, _separator_right] = Layout::horizontal([Constraint::Min(1), Constraint::Length(1), Constraint::Min(1)])
            .areas(area_main_separator);

        let vertical_separator = Block::default().borders(Borders::RIGHT).border_type(BorderType::Double);
        vertical_separator.render_ref(separator_middle, buf);

        let ql = queue_list(&self.theme, &self.queue_items);
        StatefulWidget::render(
            ql,
            area_main_right,
            buf,
            &mut ratatui::widgets::ListState::default().with_selected(self.queue_items.selected_song_index())
        );

    }
}

fn create_areas(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let [area_top, area_main] = Layout::vertical([Constraint::Length(2), Constraint::Min(1)])
        .horizontal_margin(2)
        .areas(area);

    let [area_main_left, area_main_separator, area_main_right] =
        Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
        .areas(area_main);

    (area_top, area_main_left, area_main_separator, area_main_right)
}

fn top_bar(theme: &Theme, current_directory: &Path, filter: &Option<String>) -> Block<'static> {
    let folder_name = current_directory
        .file_name()
        .map(|s| s.to_str())
        .flatten()
        .map(String::from)
        .unwrap_or("".to_string());

    let browser_title = match filter {
        Some(filter) => Line::from(vec![
            Span::styled("Search: ", Style::default()),
            Span::styled(filter.clone(), Style::default().fg(theme.search)),
        ]),
        _ => Line::from(folder_name),
    };

    let top_bar = Block::default()
        .borders(Borders::NONE)
        .title(browser_title)
        .title_alignment(Alignment::Left)
        .title_position(Position::Top)
        .title_style(Style::new().bg(theme.background).fg(theme.foreground));

    top_bar
}

fn file_list(theme: &Theme, items: &StatefulList<String>, filter: &Option<String>) -> List<'static> {
    let browser_items: Vec<ratatui::widgets::ListItem> = items
        .items()
        .iter()
        .map(|i| {
            let fg = match filter.as_ref() {
                Some(s) if i.to_lowercase().contains(&s.to_lowercase()) => theme.search,
                _ => Color::Reset,
            };
            ratatui::widgets::ListItem::new(Text::from(i.to_owned())).style(Style::default().fg(fg))
        })
        .collect();

    let browser_list = List::new(browser_items)
        .style(Style::default().fg(theme.foreground))
        .highlight_style(
            Style::default()
                .bg(theme.highlight_background)
                .fg(theme.highlight_foreground)
                .add_modifier(Modifier::BOLD),
        )
        .scroll_padding(0)
        .highlight_symbol("");

    browser_list
}

fn queue_list<'a>(theme: &Theme, queue_items: &Queue) -> List<'a> {
    let queue_items: Vec<String> = queue_items.songs().iter().map(ui::song_to_string).collect();

    let queue_list = List::new(queue_items)
        .style(Style::default().fg(theme.foreground))
        .highlight_style(
            Style::default()
                .bg(theme.highlight_background)
                .fg(theme.highlight_foreground)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");

    queue_list
}
