use std::error::Error;
use std::sync::{Arc, mpsc::Sender, atomic::{AtomicBool, Ordering}};

use async_std::task;
use mpris_server;

pub async fn run_mpris(
    play_pause: Sender<()>,
    quit: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    let player = mpris_server::Player::builder("com.tarocodes.jolteon")
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

    loop {
        task::sleep(std::time::Duration::from_secs(1)).await;
        if quit.load(Ordering::Relaxed) {
            break;
        }
    }

    Ok(())
}