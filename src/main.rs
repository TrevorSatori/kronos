mod app;
mod config;
mod constants;
mod cue;
mod extensions;
mod file_browser;
mod mpris;
mod player;
mod quit_future;
mod state;
mod structs;
mod term;
mod toml;
mod ui;

use std::error::Error;
use std::io::stdout;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use flexi_logger::{DeferredNow, FileSpec, Logger, WriteMode};
use futures::{
    future::FutureExt, // for `.fuse()`
    pin_mut,
    select,
};
use log::{debug, error, info, Record};

use crate::{app::App, mpris::run_mpris, quit_future::Quit, term::reset_terminal};

pub enum Command {
    PlayPause,
    Next,
}

pub fn log_format(w: &mut dyn std::io::Write, _now: &mut DeferredNow, record: &Record) -> Result<(), std::io::Error> {
    write!(w, "{: <12}", thread::current().name().unwrap_or("<unnamed>"),)?;

    write!(w, "{}", record.args())
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    set_panic_hook();

    let _logger = Logger::try_with_str("jolteon=debug")?
        .format(log_format)
        .log_to_file(FileSpec::default().suppress_timestamp())
        .write_mode(WriteMode::Direct)
        .use_utc()
        .start()?;

    info!("Starting");

    let (player_command_sender, player_command_receiver) = channel();

    debug!("Starting mpris and player");

    let task_player = run_player(player_command_receiver).fuse();
    let task_mpris = run_mpris(player_command_sender).fuse();

    pin_mut!(task_player, task_mpris);

    debug!("Awaiting mpris and player tasks");
    select! {
        _ = task_player => (),
        _ = task_mpris => (),
    }

    debug!("Resetting terminal");
    reset_terminal(&mut stdout());

    debug!("kthxbye");
    Ok(())
}

fn run_player(player_command_receiver: Receiver<Command>) -> Quit {
    let quit = Quit::new();
    let quit_state = quit.state();

    thread::spawn(move || {
        let mut app = App::new(player_command_receiver);
        app.start().unwrap_or_else(|err| error!("app.start error :( \n{:#?}", err));
        quit_state.lock().unwrap().complete();
    });

    quit
}

fn set_panic_hook() {
    debug!("set_panic_hook");
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = reset_terminal(&mut stdout());
        original_hook(panic_info);
    }));
}
