//! Reception of the [`HelloMessage`].

use std::time::Duration;

use futures_util::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::Instant;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

/// Errors encountered while receiving the [`HelloMessage`].
#[derive(thiserror::Error, Debug)]
pub enum HelloUpdateError {
    /// This error happens when a poll to a [`WebSocketStream`] returns an error.
    #[error("Error at the websocket layer : {0}")]
    ConnectionError(#[from] tungstenite::Error),

    /// This error happens when a poll to a [`WebSocketStream`] returns [`None`], or that the connection has been
    /// closed.
    #[error("Connection closed or lost")]
    ConnectionLost,

    /// This error happens when the deserialization of the binary data received fails.
    #[error("Parsing failed : {0:?}")]
    ParsingFailed(#[from] ciborium::de::Error<<&'static [u8] as ciborium_io::Read>::Error>),

    /// This error happens when the client sends any message type other than [`Message::Ping`] and [`Message::Binary`].
    #[error("Received a wrong websocket message type")]
    ProtocolViolation,

    /// This error indicates that the client took long enough to send the expected [`HelloMessage`] that it is
    /// considered an error.
    #[error("Didn't receive a binary message within 5 seconds")]
    Timeout,
}

/// Names of the game modes for their [`u8`] values as described in the protocol specification.
pub enum GameModes {
    MatchMadeRemote1v1,
    Local1v1,
}

impl From<GameModes> for u8 {
    fn from(value: GameModes) -> Self {
        match value {
            GameModes::MatchMadeRemote1v1 => 0,
            GameModes::Local1v1 => 1,
        }
    }
}

/// Structure representing the Hello Message as introduced in the Protocol Version 1.
pub struct HelloMessage {
    pub proto_version: u8,
    pub id: String,
    pub game_mode: u8,
}

impl HelloMessage {
    fn new(proto_version: u8, id: String, game_mode: u8) -> Self {
        HelloMessage {
            proto_version,
            id,
            game_mode,
        }
    }
}

/// Wait for the client to send the [`HelloMessage`].
///
/// Answers all pings with pongs, until either timing out, receiving some kind of error or erroneous message, or the
/// expected [`HelloMessage`].
pub async fn receive_hello_message<S>(
    websocket: &mut WebSocketStream<S>,
) -> Result<HelloMessage, HelloUpdateError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    //Set up timing out after 5 secs.
    let timeout_instant = Instant::now() + Duration::from_secs(5);
    let mut timeout_result = tokio::time::timeout_at(timeout_instant, websocket.next()).await;

    //Answer all pings, until anything different happens.
    while let Ok(Some(Ok(Message::Ping(_)))) = timeout_result {
        timeout_result = tokio::time::timeout_at(timeout_instant, websocket.next()).await;
    }

    //Process the different message.
    match timeout_result {
        Ok(Some(Ok(Message::Binary(msg)))) => match ciborium::from_reader(msg.as_slice()) {
            Ok((proto_version, id, game_mode)) => {
                Ok(HelloMessage::new(proto_version, id, game_mode))
            }
            Err(e) => Err(e.into()),
        },
        Ok(Some(Ok(_))) => Err(HelloUpdateError::ProtocolViolation),
        Ok(Some(Err(tungstenite::Error::ConnectionClosed))) | Ok(None) => {
            Err(HelloUpdateError::ConnectionLost)
        }
        Ok(Some(Err(e))) => Err(HelloUpdateError::ConnectionError(e)),
        Err(_) => Err(HelloUpdateError::Timeout),
    }
}
