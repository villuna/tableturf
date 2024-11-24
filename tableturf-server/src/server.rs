use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use color_eyre::eyre::{eyre, Context};
use futures::{SinkExt, StreamExt};
use tableturf::protocol::{ClientMessage, PublicPlayerInfo, ServerMessage};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{unbounded_channel, UnboundedSender},
        Mutex,
    },
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info, instrument, warn};

use crate::game::{handle_game, GameEvent};

/// Struct that wraps a connection to a client and handles transforming messages to/from json
#[derive(Debug)]
pub struct ClientConnection {
    inner: Mutex<Framed<TcpStream, LinesCodec>>,
}

impl ClientConnection {
    /// Create a new client connection.
    pub fn new(socket: TcpStream) -> Self {
        Self {
            inner: Mutex::new(Framed::new(socket, LinesCodec::new())),
        }
    }

    /// Recieve a message from the connected client. If the client disconnected, this will return
    /// Ok(None). If there was some unexpected error, will return an Err variant.
    pub async fn next(&self) -> color_eyre::Result<Option<ClientMessage>> {
        let line = self.inner.lock().await.next().await;

        match line {
            None => Ok(None),
            Some(line) => {
                let line = line?;
                let msg = serde_json::from_str(&line)?;
                Ok(msg)
            }
        }
    }

    /// Send a server message to the client, encoded as JSON and line terminated.
    pub async fn send(&self, msg: &ServerMessage) -> color_eyre::Result<()> {
        self.inner.lock().await.send(&serde_json::to_string(msg)?).await?;
        Ok(())
    }
}

pub type ClientId = SocketAddr;

/// The global state for the server, to be shared among all client connections.
///
/// Important: To avoid deadlocks, when acquiring multiple of these mutexes at the same time,
/// always lock in the order that they are defined here. If you always lock with a consistent
/// order, there will be no deadlocks.
#[derive(Debug, Default)]
pub struct SharedState {
    // TODO: replace this with some more sophisticated matchmaking
    /// The current client thread that is waiting for matchmaking
    pub players: Mutex<HashMap<ClientId, PublicPlayerInfo>>,
    pub channels: Mutex<HashMap<ClientId, UnboundedSender<GameEvent>>>,
    pub hotseat: Mutex<Option<ClientId>>,
}

impl SharedState {
    fn new() -> Self {
        Self::default()
    }

    // Handles a player disconnect by removing any of their data from the global state.
    async fn remove_connection(&self, addr: ClientId) {
        let mut players = self.players.lock().await;
        let mut channels = self.channels.lock().await;
        let mut hotseat = self.hotseat.lock().await;

        players.remove(&addr);
        channels.remove(&addr);
        hotseat.take_if(|a| *a == addr);
    }
}

/// Continually accepts connections from clients, spawning a new task that handles the client in
/// parallel.
async fn mainloop(listener: TcpListener) -> color_eyre::Result<()> {
    let shared = Arc::new(SharedState::new());

    loop {
        let (socket, addr) = listener.accept().await?;
        let shared_cpy = Arc::clone(&shared);
        tokio::spawn(async move {
            info!("Client connected");

            let connection = ClientConnection::new(socket);

            match handle_client(connection, addr, Arc::clone(&shared_cpy)).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{e:?}");
                }
            }

            shared_cpy.remove_connection(addr).await;
        });
    }
}

/// Runs the server on the given IP address.
pub async fn run(address: &str) -> color_eyre::Result<()> {
    let listener = TcpListener::bind(address)
        .await
        .wrap_err(format!("Failed to listen on given address {address:?}"))?;

    info!("Server running on address {address:?}");

    // Gracefully shutdown the mainloop if ctrl c was pressed
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            // Graceful shutdown goes here
            info!("Shutting down...");
        },
        res = mainloop(listener) => return res,
    }

    Ok(())
}

#[derive(Clone, Debug)]
enum ClientState {
    InLobby,
    Matchmaking,
}

/// Async task which handles a client connection.
#[instrument(skip(connection, shared_state))]
async fn handle_client(
    connection: ClientConnection,
    addr: SocketAddr,
    shared_state: Arc<SharedState>,
) -> color_eyre::Result<()> {
    let mut state = ClientState::InLobby;
    let connection = Arc::new(connection);

    // Get the player info
    let Some(ClientMessage::HelloServer { info }) = connection.next().await? else {
        warn!("Client did not say hello. Rude! Disconnecting (Client is not following protocol)");
        return Ok(());
    };

    connection.send(&ServerMessage::HelloClient).await?;
    info!("Player {:?} has joined the lobby", info.name);

    let (tx, mut rx) = unbounded_channel();

    shared_state.players.lock().await.insert(addr, info.clone());
    shared_state.channels.lock().await.insert(addr, tx);

    // Continually poll for either messages from the client or events from other parts of the
    // server.
    loop {
        tokio::select! {
            // If the client has sent us a new message
            msg = connection.next() => {
                let Some(msg) = msg? else {
                    // Client has closed the connection
                    break;
                };

                info!("Client {:?} sent message: {msg:?}", info.name);

                match (&msg, &state) {
                    (ClientMessage::FindGame, ClientState::InLobby) => {
                        state = ClientState::Matchmaking;
                        let mut hotseat = shared_state.hotseat.lock().await;

                        match *hotseat {
                            None => {
                                info!("Nobody is on the hotseat, so I'm siting down");
                                *hotseat = Some(addr);
                            }
                            Some(opp) => {
                                *hotseat = None;
                                drop(hotseat);

                                // Create a new task to handle the new game. It'll inform both
                                // client tasks that the game has started, and go from there.
                                tokio::spawn(handle_game(Arc::clone(&shared_state), [addr, opp]));
                            }
                        }
                    }

                    _ => {
                        // When the client breaks protocol, we will ignore it to allow things like sending
                        // the same message twice. But log it just to be sure.
                        warn!("Client sent message that doesn't align with protocol ({msg:?}).");
                    }
                }
            },

            Some(ev) = rx.recv() => {
                match ev {
                    GameEvent::MatchFound(tx) if matches!(state, ClientState::Matchmaking) => {
                        if tx.send(Arc::clone(&connection)).is_err() {
                            error!("Couldn't send connection over to the game handler!");
                            state = ClientState::InLobby;
                            // TODO tell client that it is game over, in the unlikely event that
                            // this happens
                        } else {
                            info!("Game has started, relinquishing connection to handler thread.");
                            info!("Now waiting for a game over event...");

                            match rx.recv().await {
                                Some(GameEvent::GameEnded) => {},
                                Some(e) => {
                                    // A hard error since this would be a wildly invalid state
                                    return Err(eyre!("Expected a GameEnded event, got something different ({e:?})"));
                                }
                                None => {
                                    error!("Game channel closed unexpectedly");
                                }
                            }

                            info!("Game has ended, returning to lobby.");
                            state = ClientState::InLobby;
                        }
                    },
                    _ => {
                        return Err(eyre!("Client task got unexpected event {ev:?} in state {state:?}"));
                    }
                }
            },
        }
    }

    info!("Closing down connection");

    Ok(())
}
