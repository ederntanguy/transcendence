//! Protocol-compliant (de)serializable structures and helper functions to communicate with clients about running
//! games.

use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;

use crate::game::Side;

/// Errors encountered while receiving an update message from the client.
#[derive(thiserror::Error, Debug)]
pub enum ClientUpdateError {
    /// This error happens when a poll to a [`WebSocketStream`] returns an error.
    #[error("Error at the websocket layer : {0}")]
    ConnectionError(#[from] tungstenite::Error),

    /// This error happens when a poll to a [`WebSocketStream`] returns [`None`], or that the connection has been
    /// closed.
    #[error("Connection closed or lost")]
    ConnectionLost,

    /// This error happens when the deserialization of the binary data received failed.
    #[error("Parsing failed : {0:?}")]
    ParsingFailed(#[from] ciborium::de::Error<<&'static [u8] as ciborium_io::Read>::Error>),

    /// This error happens when the client sends any message type other than [`Message::Ping`] and [`Message::Binary`].
    #[error("Received a wrong websocket message type")]
    ProtocolViolation,
}

/// Process the output of a poll on the given [`WebSocketStream`]. Handle [`ClientUpdateError`]s, and - if it was not a
/// ping - return the deserialized part of the update message that matters : the player pad's movement.
pub fn parse_gm0_input_message(
    msg: Option<Result<Message, tungstenite::Error>>,
) -> Result<Option<i8>, ClientUpdateError> {
    match msg {
        Some(Ok(Message::Ping(_))) => Ok(None),
        Some(Ok(Message::Binary(b))) => match ciborium::from_reader(b.as_slice()) {
            Ok((delta,)) => {
                if -1 <= delta && delta <= 1 {
                    Ok(Some(delta))
                } else {
                    Err(ClientUpdateError::ProtocolViolation)
                }
            }
            Err(e) => Err(e.into()),
        },
        Some(Ok(_)) => Err(ClientUpdateError::ProtocolViolation),
        Some(Err(tungstenite::Error::ConnectionClosed)) | None => {
            Err(ClientUpdateError::ConnectionLost)
        }
        Some(Err(e)) => Err(ClientUpdateError::ConnectionError(e)),
    }
}

pub fn parse_gm1_input_message(
    msg: Option<Result<Message, tungstenite::Error>>,
) -> Result<Option<(i8, i8)>, ClientUpdateError> {
    match msg {
        Some(Ok(Message::Ping(_))) => Ok(None),
        Some(Ok(Message::Binary(b))) => match ciborium::from_reader(b.as_slice()) {
            Ok((left_movement, right_movement))
                if -1 <= left_movement
                    && left_movement <= 1
                    && -1 <= right_movement
                    && right_movement <= 1 =>
            {
                Ok(Some((left_movement, right_movement)))
            }
            Ok(_) => Err(ClientUpdateError::ProtocolViolation),
            Err(e) => Err(e.into()),
        },
        Some(Ok(_)) => Err(ClientUpdateError::ProtocolViolation),
        Some(Err(tungstenite::Error::ConnectionClosed)) | None => {
            Err(ClientUpdateError::ConnectionLost)
        }
        Some(Err(e)) => Err(ClientUpdateError::ConnectionError(e)),
    }
}

/// Enum wrapping the various server-to-client messages introduced in the Protocol Version pre-1.
#[derive(Copy, Clone)]
pub enum ServerToClientMessage {
    PositionUpdate(PositionUpdateMessage),
    PointScored(PointScoredMessage),
    GameDone(GameCompletedMessage),
}

impl From<ServerToClientMessage> for Vec<u8> {
    fn from(value: ServerToClientMessage) -> Self {
        match value {
            ServerToClientMessage::PositionUpdate(msg) => msg.into(),
            ServerToClientMessage::PointScored(msg) => msg.into(),
            ServerToClientMessage::GameDone(msg) => msg.into(),
        }
    }
}

/// Structure representing the Position Update Message as introduced in the Protocol Version pre-1.
#[derive(Copy, Clone)]
pub struct PositionUpdateMessage {
    msg_id: u8,
    l_pad_y: f64,
    r_pad_y: f64,
    ball_x: f64,
    ball_y: f64,
}

impl PositionUpdateMessage {
    pub fn new(l_pad_y: f64, r_pad_y: f64, ball_x: f64, ball_y: f64) -> PositionUpdateMessage {
        PositionUpdateMessage {
            msg_id: 0,
            l_pad_y,
            r_pad_y,
            ball_x,
            ball_y,
        }
    }
}

impl From<PositionUpdateMessage> for Vec<u8> {
    fn from(value: PositionUpdateMessage) -> Self {
        let mut bytes = Vec::new();
        ciborium::into_writer(
            &(
                value.msg_id,
                value.l_pad_y,
                value.r_pad_y,
                value.ball_x,
                value.ball_y,
            ),
            &mut bytes,
        )
        .expect("Could not serialize a PositionUpdateMessage instance.");
        bytes
    }
}

/// Structure representing the Point Scored Message as introduced in the Protocol Version pre-1.
#[derive(Copy, Clone)]
pub struct PointScoredMessage {
    msg_id: u8,
    side: u8,
    l_pad_y: f64,
    r_pad_y: f64,
    ball_x: f64,
    ball_y: f64,
}

impl PointScoredMessage {
    pub fn new(
        side: Side,
        l_pad_y: f64,
        r_pad_y: f64,
        ball_x: f64,
        ball_y: f64,
    ) -> PointScoredMessage {
        PointScoredMessage {
            msg_id: 1,
            side: side.into(),
            l_pad_y,
            r_pad_y,
            ball_x,
            ball_y,
        }
    }
}

impl From<PointScoredMessage> for Vec<u8> {
    fn from(value: PointScoredMessage) -> Self {
        let mut bytes = Vec::new();
        ciborium::into_writer(
            &(
                value.msg_id,
                value.side,
                value.l_pad_y,
                value.r_pad_y,
                value.ball_x,
                value.ball_y,
            ),
            &mut bytes,
        )
        .expect("Could not serialize a PointScoredMessage instance.");
        bytes
    }
}

/// Structure representing the Game Completed Message as introduced in the Protocol Version pre-1.
#[derive(Copy, Clone)]
pub struct GameCompletedMessage {
    msg_id: u8,
    side: u8,
}

impl GameCompletedMessage {
    pub fn new(side: Side) -> GameCompletedMessage {
        GameCompletedMessage {
            msg_id: 2,
            side: side.into(),
        }
    }
}

impl From<GameCompletedMessage> for Vec<u8> {
    fn from(value: GameCompletedMessage) -> Self {
        let mut bytes = Vec::new();
        ciborium::into_writer(&(value.msg_id, value.side), &mut bytes)
            .expect("Could not serialize a GameDoneMessage instance.");
        bytes
    }
}

/// Structure representing the Game Aborted Message as introduced in the Protocol Version pre-2.
#[derive(Copy, Clone)]
pub struct GameAbortedMessage {
    msg_id: u8,
}

impl GameAbortedMessage {
    pub fn new() -> GameAbortedMessage {
        GameAbortedMessage { msg_id: 3 }
    }
}

impl From<GameAbortedMessage> for Vec<u8> {
    fn from(value: GameAbortedMessage) -> Self {
        let mut bytes = Vec::new();
        ciborium::into_writer(&(value.msg_id,), &mut bytes)
            .expect("Could not serialize a GameAbortedMessage instance.");
        bytes
    }
}
