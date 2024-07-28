mod app;
mod config;
mod state;
mod ui;
mod helpers;
pub mod constants;

use std::{error::Error, io, time::{Duration, Instant}};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
        execute,
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
    },
    Terminal,
};

use app::{App, InputMode};
use config::Config;
use state::load_state;
use ui::render_ui;

fn main() -> Result<(), Box<dyn Error>> {
    let state = load_state();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_secs(1);
    let app = App::new(state.last_visited_path);
    let cfg = Config::new();

    let res = run_app(&mut terminal, app, cfg, tick_rate);

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    disable_raw_mode()?;
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
        terminal.draw(|f| render_ui(f, &mut app, &cfg))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode() {
                    InputMode::Browser => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('p') | KeyCode::Char(' ') => app.music_handle.play_pause(),
                        KeyCode::Char('g') => app.music_handle.skip(),
                        KeyCode::Char('a') => {
                            app.queue_items.add(app.get_selected_browser_item());
                            app.browser_items.next();
                        },
                        KeyCode::Enter => app.evaluate(),
                        KeyCode::Backspace => app.backpedal(),
                        KeyCode::Down | KeyCode::Char('j') => app.browser_items.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.browser_items.previous(),
                        KeyCode::PageUp => app.browser_items.previous_by(5),
                        KeyCode::PageDown => app.browser_items.next_by(5),
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
                        },
                        KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                            app.set_input_mode(InputMode::BrowserFilter);
                        },
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
                    InputMode::BrowserFilter => app.handle_browser_filter_key_events(key),
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
