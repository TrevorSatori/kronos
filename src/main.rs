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
mod player;
mod cue;

use std::error::Error;
use std::io::stdout;
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
    set_panic_hook();

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

fn set_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = reset_terminal(&mut stdout());
        original_hook(panic_info);
    }));
}
