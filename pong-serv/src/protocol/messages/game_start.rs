//! Protocol-compliant serializable structures to communicate about games starting.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::game::Side;

/// Structure representing the Game Mode 0 Start Message as introduced in the Protocol Version pre-1.
#[derive(Clone)]
pub struct GameMode0StartMessage {
    enemy_username: String,
    side: u8,
    starting_time: u64,
}

impl GameMode0StartMessage {
    /// Create a new [`GameMode0StartMessage`], by converting the given `starting_time` to what is described in the
    /// Protocol.
    pub fn new(enemy_username: &str, side: Side, starting_time: SystemTime) -> Self {
        Self {
            enemy_username: String::from(enemy_username),
            side: u8::from(side),
            starting_time: starting_time_from_system_time(starting_time),
        }
    }
}

impl From<GameMode0StartMessage> for Vec<u8> {
    fn from(value: GameMode0StartMessage) -> Self {
        let mut bytes = Vec::new();
        ciborium::into_writer(
            &(value.enemy_username, value.side, value.starting_time),
            &mut bytes,
        )
        .expect("Could not serialize a GameMode0StartMessage instance.");
        bytes
    }
}

/// Structure representing the Game Mode 1 Start Message as introduced in the Protocol Version pre-3.
pub struct GameMode1StartMessage {
    starting_time: u64,
}

impl GameMode1StartMessage {
    /// Create a new [`GameMode1StartMessage`], by converting the given `starting_time` to what is described in the
    /// Protocol.
    pub fn new(starting_time: SystemTime) -> Self {
        Self {
            starting_time: starting_time_from_system_time(starting_time),
        }
    }
}

impl From<GameMode1StartMessage> for Vec<u8> {
    fn from(value: GameMode1StartMessage) -> Self {
        let mut bytes = Vec::new();
        ciborium::into_writer(&(value.starting_time,), &mut bytes)
            .expect("Could not serialize a GameMode1StartMessage instance.");
        bytes
    }
}

/// Turn a system time to a u64 amount of milliseconds since the Unix Epoch, all in UTC.
fn starting_time_from_system_time(system_time: SystemTime) -> u64 {
    system_time
        .duration_since(UNIX_EPOCH)
        .expect("Current system time is before the Unix Epoch.")
        .as_millis()
        .try_into()
        .expect(
            "We're too far in the future, deprecate the v1 communication protocol \
            (and this code xd <- this means funny)",
        )
}
