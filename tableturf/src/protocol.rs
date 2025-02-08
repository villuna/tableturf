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
    MatchFound {
        opp_info: PublicPlayerInfo,
        player_id: PlayerId,
    },
    StartWithTimeout {
        timeout: u32,
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PublicPlayerInfo {
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    HelloServer { info: PublicPlayerInfo },
    FindGame,
    Ready,
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
