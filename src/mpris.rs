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

    player.connect_play_pause({
        let player_command_sender = player_command_sender.clone();
        move |_player| {
            if let Err(err) = player_command_sender.send(Command::PlayPause) {
                error!("mpris: Failed to send play_pause! {:?}", err);
            }
        }
    });

    player.connect_next({
        let player_command_sender = player_command_sender.clone();
        move |_player| {
            if let Err(err) = player_command_sender.send(Command::Next) {
                error!("mpris: Failed to send next! {:?}", err);
            }
        }
    });

    player.run().await;

    Ok(())
}
