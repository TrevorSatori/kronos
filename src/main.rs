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
    let app = App::new(state.last_visited_path, state.queue_items.unwrap_or(vec![]));
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

        app.auto_play(); // TODO: hook into Sink's sleep_until_end

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode() {
                    InputMode::Browser => app.handle_browser_key_events(key),
                    InputMode::Queue => app.handle_queue_key_events(key),
                    InputMode::Controls => app.handle_help_key_events(key),
                    InputMode::BrowserFilter => app.handle_browser_filter_key_events(key),
                }
            }
        }

        if app.must_quit {
            break;
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    app.save_state();

    Ok(())
}
