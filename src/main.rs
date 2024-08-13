mod app;
mod config;
pub mod constants;
mod helpers;
mod state;
mod ui;

use std::error::Error;
use std::io::stdout;
use std::panic::PanicInfo;
use std::sync::{Arc, mpsc::{channel, Receiver, Sender}};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use async_std::task;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mpris_server;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    app::App,
    state::{load_state, save_state},
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
    let state = load_state();

    let mut terminal = set_terminal()?;
    let mut app = App::new(state.last_visited_path, state.queue_items);
    let state = app.start(&mut terminal, play_pause, quit)?;

    save_state(state)?;

    reset_terminal(terminal.backend_mut());
    terminal.show_cursor()?;

    Ok(())
}

async fn run_mpris(
    play_pause: Sender<()>,
    quit: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    let player = mpris_server::Player::builder("com.tarocodes.brock")
        .can_play(true)
        .can_pause(true)
        .can_go_next(true)
        .build()
        .await?;

    player.connect_play_pause(move |_player| {
        if let Err(err) = play_pause.send(()) {
            eprintln!("Failed to send play_pause! {:?}", err);
        }
    });

    player.connect_next(|_player| {
        eprintln!("next");
    });

    async_std::task::spawn_local(player.run());

    player.set_can_play(false).await?;
    player.seeked(mpris_server::Time::from_millis(1000)).await?;

    // let mut reader = crossterm::event::EventStream::new();
    // let mut event = reader.next().fuse();

    loop {
        task::sleep(std::time::Duration::from_secs(1)).await;
        if quit.load(Ordering::Relaxed) {
            break;
        }
    }

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
    execute!(writer, LeaveAlternateScreen, DisableMouseCapture).unwrap_or_else(|e| {
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
/// `stty isig icanon iexten opost ixon icrnl` should fix it.
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
