mod server;
use color_eyre::eyre::Context;
use futures::SinkExt;
pub use server::run;
use server::ClientConnection;
use tableturf::protocol::ServerMessage;

/// Send a ServerMessage to the socket, formatted as JSON and terminated by a newline.
pub async fn send_message(
    connection: &mut ClientConnection,
    msg: &ServerMessage,
) -> color_eyre::Result<()> {
    color_eyre::install()?;

    connection
        .send(serde_json::to_string(msg).wrap_err("Failed to serialize message as json")?)
        .await
        .wrap_err("Failed to send message to client")?;

    Ok(())
}
