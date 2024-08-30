use std::error::Error;
use std::sync::mpsc::Sender;

use log::error;
use mpris_server;

use crate::Command;

pub async fn run_mpris(player_command_sender: Sender<Command>) -> Result<(), Box<dyn Error>> {
    let player = mpris_server::Player::builder("com.taro-codes.jolteon")
        .can_play(true)
        .can_pause(true)
        .can_go_next(true)
        .build()
        .await?;

    let play_pause = player_command_sender.clone();
    player.connect_play_pause(move |_player| {
        if let Err(err) = play_pause.send(Command::PlayPause) {
            error!("Failed to send play_pause! {:?}", err);
        }
    });

    let next = player_command_sender.clone();
    player.connect_next(move |_player| {
        if let Err(err) = next.send(Command::Next) {
            error!("Failed to send next! {:?}", err);
        }
    });

    player.run().await;

    Ok(())
}
