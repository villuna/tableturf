use std::sync::Arc;

use color_eyre::eyre::OptionExt;
use tableturf::protocol::{PlayerId, ServerMessage};
use tokio::sync::{oneshot, mpsc};
use tracing::{error, info, instrument};

use crate::server::{ClientConnection, ClientId, SharedState};

#[derive(Debug)]
pub enum GameEvent {
    /// Signals to a matchmaking client that it has found a game.
    MatchFound(oneshot::Sender<Arc<ClientConnection>>),
    /// Signals to an in-game client that the game is over and it shoudld return to the lobby.
    GameEnded,
}

#[instrument(skip(shared_state))]
pub async fn handle_game(shared_state: Arc<SharedState>, players: [ClientId; 2]) {
    let (mut tx1, mut tx2) = {
        let channels = shared_state.channels.lock().await;

        (
            channels.get(&players[0]).ok_or_eyre("Channel not found").unwrap().clone(),
            channels.get(&players[1]).ok_or_eyre("Channel not found").unwrap().clone()
        )
    };

    match handle_game_inner(shared_state, (&mut tx1, &mut tx2), players).await {
        Ok(()) => {},
        Err(e) => error!("Game handler task encountered error: {e:?}"),
    }

    info!("Game handler closing down");

    let _ = tx1.send(GameEvent::GameEnded);
    let _ = tx2.send(GameEvent::GameEnded);
}

async fn handle_game_inner(
    shared_state: Arc<SharedState>,
    (tx1, tx2): (&mut mpsc::UnboundedSender<GameEvent>, &mut mpsc::UnboundedSender<GameEvent>),
    players: [ClientId; 2]
) -> color_eyre::Result<()> {
    let (gtx1, rx1) = oneshot::channel();
    let (gtx2, rx2) = oneshot::channel();

    tx1.send(GameEvent::MatchFound(gtx1))?;
    tx2.send(GameEvent::MatchFound(gtx2))?;

    // Wait for the clients to send over their connections
    let (connection1, connection2) = tokio::join!(rx1, rx2);
    let (connection1, connection2) = (connection1?, connection2?);

    let (info1, info2) = {
        let infos = shared_state.players.lock().await;
        (infos.get(&players[0]).unwrap().clone(), infos.get(&players[1]).unwrap().clone())
    };

    connection1.send(&ServerMessage::MatchFound { opp_info: info2.clone(), player_id: PlayerId::P1 }).await?;
    connection2.send(&ServerMessage::MatchFound { opp_info: info1.clone(), player_id: PlayerId::P2 }).await?;

    loop {
        tokio::select! {
            msg = connection1.next() => {
                let Some(msg) = msg? else { 
                    info!("P1 \"{}\" disconnected unexpectedly", info1.name);
                    connection2.send(&ServerMessage::OpponentDisconnected).await?;
                    break;
                };
                info!("P1 sent event {msg:?}");
            },
            msg = connection2.next() => {
                let Some(msg) = msg? else { 
                    info!("P2 \"{}\" disconnected unexpectedly", info2.name);
                    connection1.send(&ServerMessage::OpponentDisconnected).await?;
                    break;
                };
                info!("P2 sent event {msg:?}");
            },
        }
    }

    Ok(())
}
