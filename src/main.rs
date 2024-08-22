mod app;
mod config;
mod constants;
mod file_browser;
mod helpers;
mod mpris;
mod quit_future;
mod state;
mod term;
mod ui;

use std::error::Error;
use std::io::stdout;
use std::panic::PanicInfo;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use futures::{
    future::FutureExt, // for `.fuse()`
    pin_mut,
    select,
};

use crate::{
    app::App,
    mpris::run_mpris,
    quit_future::Quit,
    state::{load_state, save_state, State},
    term::reset_terminal,
};

pub enum Command {
    PlayPause,
    Next,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::panic::set_hook(Box::new(on_panic));

    let (player_command_sender, player_command_receiver) = channel();

    let task_player = run_player(player_command_receiver).fuse();
    let task_mpris = run_mpris(player_command_sender).fuse();

    pin_mut!(task_player, task_mpris);

    select! {
        _ = task_player => (),
        _ = task_mpris => (),
    }

    reset_terminal(&mut stdout());

    Ok(())
}

fn run_player(player_command_receiver: Receiver<Command>) -> Quit {
    let quit = Quit::new();
    let quit_state = quit.state();
    let state = load_state().unwrap_or(State::default());

    thread::spawn(move || {
        let mut app = App::new(state.last_visited_path, state.queue_items, player_command_receiver);

        match app.start() {
            Ok(state) => save_state(&state).unwrap(),
            Err(err) => eprintln!("error :( {:?}", err),
        }

        quit_state.lock().unwrap().complete();
    });

    quit
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
