use std::error::Error;
use std::sync::{Arc, mpsc::Sender, atomic::{AtomicBool, Ordering}};

use async_std::task;
use mpris_server;

use crate::Command;

pub async fn run_mpris(
    player_command_sender: Sender<Command>,
    quit: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    let player = mpris_server::Player::builder("com.taro-codes.jolteon")
        .can_play(true)
        .can_pause(true)
        .can_go_next(true)
        .build()
        .await?;

    let play_pause = player_command_sender.clone();
    player.connect_play_pause(move |_player| {
        if let Err(err) = play_pause.send(Command::PlayPause) {
            eprintln!("Failed to send play_pause! {:?}", err);
        }
    });

    let next = player_command_sender.clone();
    player.connect_next(move |_player| {
        if let Err(err) = next.send(Command::Next) {
            eprintln!("Failed to send next! {:?}", err);
        }
    });

    async_std::task::spawn_local(player.run());

    loop {
        task::sleep(std::time::Duration::from_secs(1)).await;
        if quit.load(Ordering::Relaxed) {
            break;
        }
    }

    Ok(())
}