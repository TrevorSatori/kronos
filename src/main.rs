mod app;
mod config;
mod state;

use std::{error::Error, io, time::{Duration, Instant}};

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode, EnableMouseCapture, DisableMouseCapture},
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
        ExecutableCommand,
        execute,
    },
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Gauge, List, ListItem, Row, Table, Tabs},
    Frame,
    Terminal,
    prelude::*,
    widgets::*,
};

use app::{App, AppTab, InputMode};
use config::Config;
use kronos::gen_funcs;
use state::load_state;

fn main() -> Result<(), Box<dyn Error>> {
    let state = load_state();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_secs(1);
    let app = App::new(state.last_visited_path);
    let cfg = Config::new();

    let res = run_app(&mut terminal, app, cfg, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    cfg: Config,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app, &cfg))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            // different keys depending on which browser tab
            if let Event::Key(key) = event::read()? {
                match app.input_mode() {
                    InputMode::Browser => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('p') | KeyCode::Char(' ') => app.music_handle.play_pause(),
                        KeyCode::Char('g') => app.music_handle.skip(),
                        KeyCode::Char('a') => app.queue_items.add(app.selected_item()),
                        KeyCode::Enter => app.evaluate(),
                        KeyCode::Backspace => app.backpedal(),
                        KeyCode::Down | KeyCode::Char('j') => app.browser_items.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.browser_items.previous(),
                        KeyCode::End => app.browser_items.select(app.browser_items.items().len() - 1),
                        KeyCode::Home => app.browser_items.select(0),
                        KeyCode::Right | KeyCode::Char('l') => {
                            app.browser_items.unselect();
                            app.set_input_mode(InputMode::Queue);
                            app.queue_items.next();
                        }
                        KeyCode::Char('-') => app.music_handle.change_volume(-0.05),
                        KeyCode::Char('+') => app.music_handle.change_volume(0.05),
                        KeyCode::Tab => {
                            app.next();
                            match app.input_mode() {
                                InputMode::Controls => app.set_input_mode(InputMode::Browser),
                                _ => app.set_input_mode(InputMode::Controls),
                            };
                        }
                        _ => {}
                    },
                    InputMode::Queue => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('p') => app.music_handle.play_pause(),
                        KeyCode::Char('g') => app.music_handle.skip(),
                        KeyCode::Enter => {
                            if let Some(i) = app.queue_items.item() {
                                app.music_handle.play(i.clone());
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.queue_items.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.queue_items.previous(),
                        KeyCode::Char('r') => app.queue_items.remove(),
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.queue_items.unselect();
                            app.set_input_mode(InputMode::Browser);
                            app.browser_items.next();
                        }
                        KeyCode::Tab => {
                            app.next();
                            match app.input_mode() {
                                InputMode::Controls => app.set_input_mode(InputMode::Browser),
                                _ => app.set_input_mode(InputMode::Controls),
                            };
                        }
                        _ => {}
                    },
                    InputMode::Controls => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('p') => app.music_handle.play_pause(),
                        KeyCode::Char('g') => app.music_handle.skip(),
                        KeyCode::Down | KeyCode::Char('j') => app.control_table.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.control_table.previous(),
                        KeyCode::Tab => {
                            app.next();
                            match app.input_mode() {
                                InputMode::Controls => app.set_input_mode(InputMode::Browser),
                                _ => app.set_input_mode(InputMode::Controls),
                            };
                        }
                        _ => {}
                    },
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    app.save_state();

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App, cfg: &Config) {
    // Total Size
    let size = f.size();

    // chunking from top to bottom, 3 gets tabs displayed, the rest goes to item layouts
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    // Main Background block, covers entire screen
    let block = Block::default().style(Style::default().bg(cfg.background()));
    f.render_widget(block, size);

    // Tab Title items collected
    let titles: Vec<Line> = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(first, Style::default().fg(cfg.highlight_background())), // CHANGE FOR CUSTOMIZATION
                Span::styled(rest, Style::default().fg(cfg.highlight_background())), // These are tab highlights, first vs rest diff colors
            ])
        })
        .collect();

    // Box Around Tab Items
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.active_tab as usize)
        .style(Style::default().fg(cfg.foreground()))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(cfg.background()),
        );
    f.render_widget(tabs, chunks[0]);

    match app.active_tab {
        AppTab::Music => music_tab(f, app, chunks[1], cfg),
        AppTab::Controls => instructions_tab(f, app, chunks[1], cfg),
    };
}

fn music_tab(frame: &mut Frame, app: &mut App, chunks: Rect, cfg: &Config) {
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
        .map(|i| ListItem::new(Text::from(i.to_owned())))
        .collect();

    let browser_list = List::new(browser_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Browser")
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

fn instructions_tab(f: &mut Frame, app: &mut App, chunks: Rect, cfg: &Config) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(chunks);

    // map header to tui object
    let header = app
        .control_table
        .header
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(cfg.highlight_foreground())));

    // Header and first row
    let header = Row::new(header)
        .style(Style::default().bg(cfg.background()).fg(cfg.foreground()))
        .height(1)
        .bottom_margin(1);

    // map items from table to Row items
    let rows = app.control_table.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let widths = [
        Constraint::Length(5),
        Constraint::Length(10),
    ];

    let t = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(cfg.foreground()).bg(cfg.background()))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(cfg.highlight_background())
                .fg(cfg.highlight_foreground()),
        )
        // .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, chunks[0], &mut app.control_table.state);
}
