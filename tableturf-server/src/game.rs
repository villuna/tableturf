use std::sync::Arc;

use color_eyre::eyre::OptionExt;
use tableturf::protocol::PlayerId;
use tokio::sync::mpsc::unbounded_channel;
use tracing::{info, instrument};

use crate::server::{ClientGameInfo, ClientId, ServerEvent, SharedState};

#[derive(Debug)]
pub enum GameEvent {
}

#[instrument(skip(shared_state))]
pub async fn handle_game(shared_state: Arc<SharedState>, players: [ClientId; 2]) -> color_eyre::Result<()> {
    let mut channels = shared_state.channels.lock().await;

    let (gtx1, mut rx1) = unbounded_channel();
    let (gtx2, mut rx2) = unbounded_channel();

    let tx1 = channels.get_mut(&players[0]).ok_or_eyre("Channel not found")?.clone();
    let tx2 = channels.get_mut(&players[1]).ok_or_eyre("Channel not found")?.clone();

    tx1.send(ServerEvent::MatchFound(ClientGameInfo { 
            player_id: PlayerId::P1,
            opponent: players[0],
            game_tx: gtx1,
        }))?;
    tx2.send(ServerEvent::MatchFound(ClientGameInfo { 
            player_id: PlayerId::P2,
            opponent: players[1],
            game_tx: gtx2,
        }))?;

    loop {
        tokio::select! {
            ev = rx1.recv() => {
                let Some(ev) = ev else { break; };
                info!("P1 sent event {ev:?}");
            },
            ev = rx2.recv() => {
                let Some(ev) = ev else { break; };
                info!("P2 sent event {ev:?}");
            },
        }
    }

    info!("Game handler closing down");

    Ok(())
}
