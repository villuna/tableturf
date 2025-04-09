use crate::cards::Deck;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PlayerId {
    P1 = 0,
    P2 = 1,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    HelloClient,
    /// Response to [`ClientMessage::Pong`]
    Pong { number: usize },
    WaitForOpponent,
    MatchFound {
        opp_info: PublicPlayerInfo,
        player_id: PlayerId,
    },
    /// Sent to a client if their opponent disconnects midgame. Depending on the stage of the game
    /// this might count as a win or a draw.
    OpponentDisconnected,
    StartWithTimeout {
        timeout: u32,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PublicPlayerInfo {
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    HelloServer { info: PublicPlayerInfo },
    /// Sent to the server every now and then to check if the server is still alive
    /// A client disconnect is detectable by the server but a server crash is undetectable by the
    /// client, hence why this is necessary.
    /// The number is randomised and we expect the server to respond with the same number so we
    /// know that e.g. we are not getting a response meant for a different user.
    Ping { number: usize },
    FindGame,
    Ready,
    ChosenDeck { deck: Deck },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_server_protocol_ser() {
        let hello = ServerMessage::HelloClient;
        let json = serde_json::to_string(&hello).unwrap();

        assert_eq!(json, r#"{"type":"hello_client"}"#.to_string());
    }

    #[test]
    fn test_client_protocol_ser() {
        let hello = ClientMessage::HelloServer {
            info: PublicPlayerInfo {
                name: "villuna".to_string(),
            },
        };
        let json = serde_json::to_string(&hello).unwrap();

        assert_eq!(json, r#"{"type":"hello_server","info":{"name":"villuna"}}"#);
    }

    #[test]
    fn test_client_protocol_de() {
        let hello = ClientMessage::HelloServer {
            info: PublicPlayerInfo {
                name: "villuna".to_string(),
            },
        };

        assert_eq!(
            serde_json::from_str::<ClientMessage>(
                r#"{"type":"hello_server","info":{"name":"villuna"}}"#
            )
            .unwrap(),
            hello,
        );

        assert_eq!(
            serde_json::from_str::<ClientMessage>(
                r#"{"info":{"name":"villuna"},"type":"hello_server"}"#
            )
            .unwrap(),
            hello,
        );
    }
}
