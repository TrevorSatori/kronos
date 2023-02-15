use std::time::{Instant, Duration};
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lib::gen_funcs;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Text, Spans, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Gauge, Tabs},
    Frame, Terminal,
};

mod lib;
use crate::lib::{app::*};
pub mod config;
use config::Config;

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
    let cfg = Config::new();

    println!("{:#?}", cfg);
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
        println!("{:?}", err)
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

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {

            // different keys depending on which browser tab
            if let Event::Key(key) = event::read()? {
                match app.get_input_mode() {
                    InputMode::Browser => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('p') | KeyCode::Char(' ') => app.music_handle.play_pause(),
                        KeyCode::Char('g') => app.music_handle.skip(),
                        KeyCode::Char('a') => app.queue_items.add(app.selected_item()),
                        KeyCode::Enter => app.evaluate(),
                        KeyCode::Backspace => app.backpedal(),
                        KeyCode::Down | KeyCode::Char('j') => app.browser_items.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.browser_items.previous(),
                        KeyCode::Right |  KeyCode::Char('l') => {
                            app.browser_items.unselect();
                            app.set_input_mode(InputMode::Queue);
                            app.queue_items.next();

                        },
                        KeyCode::Tab => app.next(),
                        _ => {}
                    },
                    InputMode::Queue => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('p') => app.music_handle.play_pause(),
                        KeyCode::Char('g') => app.music_handle.skip(),
                        KeyCode::Enter => app.music_handle.play(app.queue_items.get_item().clone()),
                        KeyCode::Down | KeyCode::Char('j') => app.queue_items.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.queue_items.previous(),
                        KeyCode::Char('r') => app.queue_items.remove(),
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.queue_items.unselect();
                            app.set_input_mode(InputMode::Browser);
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


fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, cfg: &Config) {
    
    // Total Size
    let size = f.size();
    let fg = cfg.get_foreground(); 
    let hfg = Color::Black;
    let hbg = Color::LightCyan;

    // chunking from top to bottom, 3 gets tabs displayed, the rest goes to item layouts
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
    .split(size);

    // Main Background block, covers entire screen 
    let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::Black));
    f.render_widget(block, size);

    
    // Tab Title items collected
    let titles = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    // Box Around Tab Items
    let tabs = Tabs::new(titles)
    .block(Block::default().borders(Borders::ALL).title("Tabs"))
    .select(app.index)
    .style(Style::default().fg(fg))
    .highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Black),
    );
    f.render_widget(tabs, chunks[0]);

    let _inner = match app.index {
        0 => music_tab(f, app,chunks[1], fg, hfg, hbg),
        1 => instructions_tab(f, app,chunks[1], fg, hfg, hbg),
        _ => unreachable!(),
    };      
}

fn music_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, chunks: Rect, fg: Color, hfg: Color, hbg: Color){
    
    // split into left / right
    let browser_queue = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
    .split(chunks);
    // f.size()

    // queue and playing sections
    let queue_playing = Layout::default()
        .direction(Direction::Vertical)
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
        .style(Style::default().fg(fg))
        .highlight_style(
            Style::default()
                .bg(hbg)
                .fg(hfg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(items, browser_queue[0], &mut app.browser_items.get_state());


    let queue_items: Vec<ListItem> = app.queue_items.get_items()
        .iter()
        .map(|i| {
            
            ListItem::new(Text::from(gen_funcs::audio_display(&i)))
        })
        .collect();
    
    let queue_title = "| Queue: ".to_owned() 
    + &app.queue_items.get_length().to_string() + " Songs |" + &app.queue_items.get_total_time();
    
    
    let queue_items = List::new(queue_items)
        .block(Block::default()
        .borders(Borders::ALL)
        .title(queue_title)
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded))
        .style(Style::default().fg(fg))
        .highlight_style(
            Style::default()
                .bg(hbg)
                .fg(hfg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(queue_items, queue_playing[0], &mut app.queue_items.get_state());

    

    let playing_title = "| ".to_owned() + &app.get_current_song() + " |";

    let playing = Gauge::default()
        .block(Block::default()
        .title(playing_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_alignment(Alignment::Center))
        .style(Style::default().fg(fg))
        .gauge_style(Style::default().fg(Color::LightCyan))
        .percent(app.song_progress());
    f.render_widget(playing, queue_playing[1]);
}

fn instructions_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, chunks: Rect, fg: Color, hfg: Color, hbg: Color){
    
    //  
    let keys = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(10), Constraint::Percentage(10),
    Constraint::Percentage(10),Constraint::Percentage(10),Constraint::Percentage(10),
    Constraint::Percentage(10),
    Constraint::Percentage(10),Constraint::Percentage(10),Constraint::Percentage(10),].as_ref())
    .split(chunks);

    let quit = Block::default().style(Style::default()
    .bg(Color::Black)
    .fg(fg))
    .title_alignment(Alignment::Center).border_type(BorderType::Rounded)
    .border_style(Style::default().fg(hfg))
    .title("QUIT - Q"); 
    f.render_widget(quit,  keys[0]);

    let pause = Block::default().style(Style::default()
    .bg(Color::Black)
    .fg(fg))
    .title_alignment(Alignment::Center).border_type(BorderType::Rounded)
    .border_style(Style::default().fg(hfg))
    .title("PAUSE / PLAY - SPACE or P");
    f.render_widget(pause,  keys[1]);

    let skip = Block::default().style(Style::default()
    .bg(Color::Black)
    .fg(fg))
    .title_alignment(Alignment::Center).border_type(BorderType::Rounded)
    .border_style(Style::default().fg(hfg))
    .title("SKIP - G");
    f.render_widget(skip,  keys[2]);

    let add = Block::default().style(Style::default()
    .bg(Color::Black)
    .fg(fg))
    .title_alignment(Alignment::Center)
    .borders(Borders::ALL)
    .border_style(Style::default().fg(hfg))
    .title("ADD TO QUEUE - A");
    f.render_widget(add,  keys[3]);

    let play = Block::default().style(Style::default()
    .bg(Color::Black)
    .fg(fg))
    .title_alignment(Alignment::Center)
    .borders(Borders::ALL)
    .border_style(Style::default().fg(hfg))
    .title("PLAY / ENTER DIRECTORY - ENTER");
    f.render_widget(play,  keys[4]);

    let pdir = Block::default().style(Style::default()
    .bg(Color::Black)
    .fg(fg))
    .title_alignment(Alignment::Center).border_type(BorderType::Rounded)
    .border_style(Style::default().fg(hfg))
    .title("PARENT DIRECTORY - BACKSPACE");
    f.render_widget(pdir,  keys[5]);

    let down = Block::default().style(Style::default()
    .bg(Color::Black)
    .fg(fg))
    .title_alignment(Alignment::Center)
    .border_type(BorderType::Rounded)
    .title("NEXT ITEM - DOWN ARROW or J");
    f.render_widget(down,  keys[6]);

    let up = Block::default().style(Style::default()
    .bg(Color::Black)
    .fg(fg))
    .title_alignment(Alignment::Center).border_type(BorderType::Rounded)
    .border_style(Style::default().fg(hfg))
    .title("PREVIOUS ITEM - UP ARROW or K");
    f.render_widget(up,  keys[7]);

    let toggle = Block::default().style(Style::default()
    .bg(Color::Black)
    .fg(fg))
    .title_alignment(Alignment::Center).border_type(BorderType::Rounded)
    .border_style(Style::default().fg(hfg))
    .title("CHANGE FOCUS - LEFT & RIGHT ARROWS or H & L KEYS");
    f.render_widget(toggle,  keys[8]);

    
}
