mod app;
mod config;
mod state;
mod ui;
mod helpers;
pub mod constants;

use std::{error::Error};
use std::io::stdout;
use std::panic::PanicInfo;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event},
        execute,
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
    },
    Terminal,
};

use app::{App};
use state::load_state;

fn main() -> Result<(), Box<dyn Error>> {
    std::panic::set_hook(Box::new(on_panic));

    let state = load_state();

    let mut terminal = set_terminal()?;
    let mut app = App::new(state.last_visited_path, state.queue_items.unwrap_or(vec![]));
    let res = app.start(&mut terminal);

    if let Err(err) = res {
        eprintln!("{:?}", err)
    }

    reset_terminal(terminal.backend_mut());

    terminal.show_cursor()?;

    Ok(())
}

fn set_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, impl Error> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn reset_terminal(writer: &mut impl std::io::Write) {
    execute!(
        writer,
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap_or_else(|e| {
        eprintln!("tried to execute(...) but couldn't :( {e}");
    });

    disable_raw_mode().unwrap_or_else(|e| {
        eprintln!("tried to disable_raw_mode but couldn't :( {e}");
    });
}

/// If our app panics after entering raw mode and before leaving it,
/// the terminal that was running our app will be left in raw mode.
/// Raw mode seems to be a concept of the standard C library and not terminal emulators themselves.
/// Crossterm calls `cfmakeraw`, which does a bunch of things.
///
/// This hook should take care of reverting it the terminal back to how it was if the app panics,
/// but if we're still left with a somewhat unusable terminal for whatever reason,
/// `stty isig icanon iexten opost ixon icrnl ` should fix it.
///
/// See `man cfmakeraw` and `man stty`.
fn on_panic(info: &PanicInfo) {
    eprintln!("panic at the disco {info}");

    // We don't have access to our instances of `stdout` and/or `backend` here,
    // but referencing `io::stdout()` every time seems to work.
    // I guess that could only fail if the stdout of the process changes while
    // the process is running... but that is an edge case bug I can live with.
    reset_terminal(&mut stdout());
}