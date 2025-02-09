use serde_json::Deserializer;
use std::{
    io::{BufReader, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    thread::JoinHandle,
};
use tableturf::protocol::{ClientMessage, ServerMessage};

struct Connection {
    thread: JoinHandle<()>,
    rx: Receiver<ServerMessage>,
    connection: Arc<TcpStream>,
}

impl Connection {
    fn new(addr: impl ToSocketAddrs) -> color_eyre::Result<Self> {
        let connection = Arc::new(TcpStream::connect(addr)?);
        let (tx, rx) = channel();

        let connection_cpy = Arc::clone(&connection);
        let thread = std::thread::spawn(move || {
            println!("Thread has started");
            let stream = Deserializer::from_reader(BufReader::new(connection_cpy.as_ref()));

            for msg in stream.into_iter() {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Error while trying to read server message ({e:?})");
                        break;
                    }
                };
                tx.send(msg).unwrap();
            }

            println!("Closing down client thread");
        });

        Ok(Self {
            thread,
            rx,
            connection,
        })
    }
}

/// An interface that handles the connection with the game server
///
/// The rest of the game will use this to check if there is a connection and send/recieve messages
/// to and from the server.
pub struct GameContext {
    connection: Option<Connection>,
}

impl GameContext {
    /// Creates a new GameContext, but does not try to connect to anything.
    pub fn new() -> Self {
        Self {
            connection: None,
        }
    }

    /// Returns whether the server is connected, i.e. a connection has been made.
    pub fn connected(&self) -> bool {
        self.connection.is_some()
    }

    /// Attempts to connect to the server if it is not already connected
    pub fn connect(&mut self, address: impl ToSocketAddrs) {
        if self.connection.is_none() {
            self.connection = Connection::new(address).ok();
        }
    }

    /// Disconnects from the server if connected
    pub fn disconnect(&mut self) {
        if let Some(connection) = self.connection.take() {
            let _ = connection.connection.shutdown(std::net::Shutdown::Both);
            let _ = connection.thread.join();
        }
    }

    // Helper function for send
    fn try_send(&mut self, msg: &ClientMessage) -> color_eyre::Result<()> {
        if let Some(connection) = &mut self.connection {
            connection
                .connection
                .as_ref()
                .write_all(serde_json::to_string(msg)?.as_bytes())?;
            connection.connection.as_ref().write(b"\n")?;
        }

        Ok(())
    }

    /// Attempts to send a message to the server. If it encounters an error, it will disconnect.
    pub fn send(&mut self, msg: &ClientMessage) -> color_eyre::Result<()> {
        let result = self.try_send(msg); 

        if result.is_err() {
            self.disconnect();
        }

        result
    }

    /// Polls the connection and returns a new message if one has been recieved.
    pub fn recv(&mut self) -> Option<ServerMessage> {
        self.connection.as_mut().and_then(|c| c.rx.try_recv().ok())
    }
}
