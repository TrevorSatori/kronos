mod app;
mod config;
mod constants;
mod helpers;
mod state;
mod ui;
mod mpris;
mod term;

use std::error::Error;
use std::io::stdout;
use std::panic::PanicInfo;
use std::sync::{Arc, mpsc::{channel, Receiver}, atomic::AtomicBool};
use std::thread;

use crate::{
    app::App,
    state::{load_state, save_state, State},
    mpris::run_mpris,
    term::{reset_terminal, set_terminal},
};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::panic::set_hook(Box::new(on_panic));

    let quit: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let (play_pause_sender, play_pause_receiver) = channel();

    run_player_thread(play_pause_receiver, quit.clone());
    run_mpris(play_pause_sender, quit.clone()).await?;

    Ok(())
}

fn run_player_thread(play_pause: Receiver<()>, quit: Arc<AtomicBool>) {
    thread::spawn(move || {
        if let Err(err) = run_player(play_pause, quit) {
            eprintln!("error :( {:?}", err);
        }
    });
}

fn run_player(play_pause: Receiver<()>, quit: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    let state = load_state().unwrap_or(State::default());

    let mut terminal = set_terminal()?;
    let mut app = App::new(state.last_visited_path, state.queue_items);
    let state = app.start(&mut terminal, play_pause, quit)?;

    save_state(&state)?;

    reset_terminal(terminal.backend_mut());
    terminal.show_cursor()?;

    Ok(())
}

/// If our app panics after entering raw mode and before leaving it,
/// the terminal that was running our app will be left in raw mode.
/// Raw mode seems to be a concept of the standard C library and not terminal emulators themselves.
/// Crossterm calls `cfmakeraw`, which does a bunch of things.
///
/// This hook should take care of reverting it the terminal back to how it was if the app panics,
/// but if we're still left with a somewhat unusable terminal for whatever reason,
/// `reset` or `stty isig icanon iexten opost ixon icrnl` should fix it.
///
/// See `man cfmakeraw` and `man stty`.
fn on_panic(info: &PanicInfo) {
    // We don't have access to our instances of `stdout` and/or `backend` here,
    // but referencing `io::stdout()` every time seems to work.
    // I guess that could only fail if the stdout of the process changes while
    // the process is running... but that is an edge case bug I can live with.
    reset_terminal(&mut stdout());

    eprintln!("{info}");
}
