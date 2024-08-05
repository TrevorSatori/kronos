mod app;
mod config;
mod state;
mod ui;
mod helpers;
pub mod constants;

use std::{error::Error, io};

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
    std::panic::set_hook(Box::new(|info| {
        // If our app panics after entering raw mode and before leaving it,
        // the terminal that was running our app will be left in raw mode.
        // Raw mode seems to be a concept of the standard C library and not terminal emulators themselves.
        // Crossterm calls `cfmakeraw`, which does a bunch of things.
        //
        // This hook should take care of reverting it the terminal back to how it was if the app panics,
        // but if we're still left with a somewhat unusable terminal for whatever reason,
        // `stty isig icanon iexten opost ixon icrnl ` should fix it.
        //
        // See https://linux.die.net/man/3/cfmakeraw, https://man7.org/linux/man-pages/man1/stty.1.html
        eprintln!("panic at the disco {info}");

        // We don't have access to our instances of `stdout` and `backend` here,
        // but referencing `io::stdout()` every time seems to work.
        // I guess that could only fail if the stdout of the process changes while
        // the process is running... but that is an edge case bug I can live with.
        reset_terminal();
    }));


    let state = load_state();

    // let a: Option<u32> = None;
    // a.unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(state.last_visited_path, state.queue_items.unwrap_or(vec![]));
    let res = app.start(&mut terminal);

    reset_terminal(); // We'd normally use terminal.backend_mut(), but whatever.

    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err)
    }

    Ok(())
}

fn reset_terminal() {
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap_or_else(|e| {
        eprintln!("tried to execute(...) but couldn't :( {e}");
    });

    disable_raw_mode().unwrap_or_else(|e| {
        eprintln!("tried to disable_raw_mode but couldn't :( {e}");
    });
}