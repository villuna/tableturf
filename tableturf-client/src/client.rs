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

pub struct GameContext {
    online: bool,
    connection: Option<Connection>,
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            online: false,
            connection: None,
        }
    }

    pub fn connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn online(&self) -> bool {
        self.online
    }

    pub fn connect(&mut self, address: impl ToSocketAddrs) {
        if !self.online {
            self.online = true;
            self.connection = Connection::new(address).ok();
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(connection) = self.connection.take() {
            let _ = connection.connection.shutdown(std::net::Shutdown::Both);
            let _ = connection.thread.join();
        }
    }

    pub fn send(&mut self, msg: &ClientMessage) -> color_eyre::Result<()> {
        if let Some(connection) = &mut self.connection {
            connection
                .connection
                .as_ref()
                .write_all(serde_json::to_string(msg)?.as_bytes())?;
            connection.connection.as_ref().write(b"\n")?;
        }

        Ok(())
    }

    pub fn recv(&mut self) -> Option<ServerMessage> {
        self.connection.as_mut().and_then(|c| c.rx.try_recv().ok())
    }
}
