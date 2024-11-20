use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use color_eyre::eyre::Context;
use futures::{SinkExt, StreamExt};
use tableturf::protocol::{ClientMessage, ServerMessage};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info, instrument, warn};

/// Struct that wraps a connection to a client and handles transforming messages to/from json
pub struct ClientConnection {
    inner: Framed<TcpStream, LinesCodec>,
}

impl ClientConnection {
    /// Create a new client connection.
    pub fn new(socket: TcpStream) -> Self {
        Self {
            inner: Framed::new(socket, LinesCodec::new()),
        }
    }

    /// Recieve a message from the connected client. If the client disconnected, this will return
    /// Ok(None). If there was some unexpected error, will return an Err variant.
    pub async fn next(&mut self) -> color_eyre::Result<Option<ClientMessage>> {
        let line = self.inner.next().await;

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
    pub async fn send(&mut self, msg: &ServerMessage) -> color_eyre::Result<()> {
        self.inner.send(&serde_json::to_string(msg)?).await?;
        Ok(())
    }
}

/// The global state for the server, to be shared among all client connections.
///
/// Important: To avoid deadlocks, when acquiring multiple of these mutexes at the same time,
/// always lock in the order that they are defined here. If you always lock with a consistent
/// order, there will be no deadlocks.
#[derive(Debug, Default)]
struct SharedState {
    // TODO: replace this with some more sophisticated matchmaking
    /// The current client thread that is waiting for matchmaking
    players: Mutex<HashSet<SocketAddr>>,
    hotseat: Mutex<Option<SocketAddr>>,
}

impl SharedState {
    fn new() -> Self {
        Self::default()
    }

    // Handles a player disconnect by removing any of their data from the global state.
    async fn remove_connection(&self, addr: SocketAddr) {
        let mut players = self.players.lock().unwrap();
        let mut hotseat = self.hotseat.lock().unwrap();

        players.remove(&addr);
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ClientState {
    InLobby,
    Matchmaking,
}

/// Async task which handles a client connection.
#[instrument(skip(connection, shared_state))]
async fn handle_client(
    mut connection: ClientConnection,
    addr: SocketAddr,
    shared_state: Arc<SharedState>,
) -> color_eyre::Result<()> {
    let mut state = ClientState::InLobby;

    // Get the player info
    let Some(ClientMessage::HelloServer { info }) = connection.next().await? else {
        warn!("Client did not say hello. Rude! Disconnecting (Client is not following protocol)");
        return Ok(());
    };

    connection.send(&ServerMessage::HelloClient).await?;
    info!("Player {:?} has joined the lobby", info.name);

    shared_state.players.lock().unwrap().insert(addr);

    while let Some(msg) = connection.next().await? {
        info!("Client {:?} sent message: {msg:?}", info.name);

        match msg {
            ClientMessage::FindGame if state == ClientState::InLobby => {
                let mut hotseat = shared_state.hotseat.lock().unwrap();

                match *hotseat {
                    None => {
                        info!("Nobody is on the hotseat, so I'm siting down");
                        *hotseat = Some(addr);
                        state = ClientState::Matchmaking;
                    }
                    Some(opp) => {
                        *hotseat = None;
                        info!("Starting game match with opponent {opp:?}");
                        // TODO: start the game
                    }
                }
            }
            _ => {
                // When the client breaks protocol, we will ignore it to allow things like sending
                // the same message twice. But log it just to be sure.
                warn!("Client sent message that doesn't align with protocol ({msg:?}).");
            }
        }
    }

    info!("Closing down connection");

    Ok(())
}
