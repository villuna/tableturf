use crate::send_message;
use color_eyre::eyre::Context;
use futures::StreamExt;
use tableturf::protocol::{ClientMessage, ServerMessage};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info, instrument, warn};

pub type ClientConnection = Framed<TcpStream, LinesCodec>;

/// Continually accepts connections from clients, spawning a new task that handles the client in
/// parallel.
async fn mainloop(listener: TcpListener) -> color_eyre::Result<()> {
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            match handle_client(socket).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{e:?}");
                }
            }
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

/// Responds to a client's message. `writer` is the write handle to the client which sent
/// the message.
#[instrument(skip(writer))]
async fn handle_message(
    writer: &mut ClientConnection,
    msg: ClientMessage,
) -> color_eyre::Result<()> {
    use ClientMessage as CM;
    match msg {
        CM::HelloServer { info } => {
            info!("Player \"{}\" said hello!", info.name);
            send_message(writer, &ServerMessage::HelloClient).await?;
        }
        CM::FindGame => {
            info!("Player is trying to find a game");
        }
    }

    Ok(())
}

/// Async task which handles a client connection.
#[instrument]
async fn handle_client(socket: TcpStream) -> color_eyre::Result<()> {
    info!("Client connected");

    let mut connection = ClientConnection::new(socket, LinesCodec::new());

    while let Some(line) = connection.next().await {
        let line = line?;
        let Ok(msg) = serde_json::from_str(&line) else {
            warn!("Client sent malformed message: {line:?}");
            break;
        };

        info!("Client sent message: {msg:?}");
        handle_message(&mut connection, msg).await?;
    }

    info!("Closing down connection");

    Ok(())
}
