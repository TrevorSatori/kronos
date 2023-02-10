use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lib::queue;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span,Spans, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, Gauge},
    Frame, Terminal,
};


mod lib;
use crate::lib::{app::*, stateful_list::*};

use std::io::BufReader;
use rodio::{Sink, Decoder, OutputStream, source::Source};
use std::ffi::OsStr;

use std::time::{Instant, Duration};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, DisableMouseCapture)?; 

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_secs(1);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}


fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {

            // different keys depending on which browser tab
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Browser => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('p') => app.play_pause(),
                        KeyCode::Char('a') => app.queue_items.add(app.selected_item()),
                        KeyCode::Enter => app.evaluate(),
                        KeyCode::Backspace => app.backpedal(),
                        KeyCode::Down | KeyCode::Char('j') => app.browser_items.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.browser_items.previous(),
                        KeyCode::Right |  KeyCode::Char('l') => {
                            app.browser_items.unselect();
                            app.input_mode = InputMode::Queue;
                            app.queue_items.next();

                        },
                        _ => {}
                    },
                    InputMode::Queue => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('p') => app.play_pause(),
                        KeyCode::Enter => app.play(app.selected_item()),
                        KeyCode::Down | KeyCode::Char('j') => app.queue_items.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.queue_items.previous(),
                        KeyCode::Char('r') => app.queue_items.remove(),
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.queue_items.unselect();
                            app.input_mode = InputMode::Browser;
                            app.browser_items.next();
                        }
                        _ => {}
                    }      
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}


fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    // split into left / right
    let browser_queue = Layout::default()
        .direction(Direction::Horizontal)
        // .margin(5)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(f.size());

    // queue and playing sections
    let queue_playing = Layout::default()
        .direction(Direction::Vertical)
        // .margin(5)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
        .split(browser_queue[1]);

    
    // convert app items to text
    let items: Vec<ListItem> = app.browser_items.get_items()
    .iter()
    .map(|i| {
        ListItem::new(Text::from(i.to_owned()))
    })
    .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL)
        .title("Browser")
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded))
        .highlight_style(
            Style::default()
                .bg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(items, browser_queue[0], &mut app.browser_items.state);

    // STORE PATHBUF IN QUEUE, DISPLAY AS STRING
    // convert queue items to text
    let queue_items: Vec<ListItem> = app.queue_items.get_items()
        .iter()
        .map(|i| {
            ListItem::new(Text::from(i.file_name().unwrap().to_str().unwrap().to_string()))
        })
        .collect();
    
    let queue_title = "QUEUE ITEMS ".to_owned() + &app.queue_items.length().to_string();
     // Create a List from all list items and highlight the currently selected one
    let queue_items = List::new(queue_items)
        .block(Block::default()
        .borders(Borders::ALL)
        .title(queue_title)
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded))
        .highlight_style(
            Style::default()
                .bg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(queue_items, queue_playing[0], &mut app.queue_items.state);

    
    let playing_title = "| ".to_owned() + &app.get_current_song() + " |";
    let playing = Gauge::default()
        .block(Block::default()
        .title(playing_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_alignment(Alignment::Center))
        .gauge_style(Style::default().fg(Color::LightCyan))
        .percent(app.song_progress());
    f.render_widget(playing, queue_playing[1]);

}

