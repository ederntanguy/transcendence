//! Encapsulation of a game state, and computation of its evolution.

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

pub(super) use done::GameResult;
pub(super) use running::RunningState;

use super::Player;

mod done;
mod running;
mod startup;

/// Current state - or stage - of a game mode 0 game.
pub(super) enum Game0State {
    Startup,
    Running(RunningState),
    Done(GameResult),
}

impl Game0State {
    /// Create a new game (mode 0) state at the initial stage of startup.
    pub(super) fn new() -> Self {
        Self::Startup
    }

    /// Try to complete the current stage to get to the next one and return it.
    ///
    /// # Error
    ///
    /// Only fails with an error if there's a disconnection during the startup period. Later disconnections are not
    /// errors, they are withdrawals.
    /// The returned error is paired with the websocket that didn't fail.
    pub(super) async fn next_state<S>(
        self,
        mut left_player: Player<S>,
        mut right_player: Player<S>,
    ) -> Result<(Self, Player<S>, Player<S>), (tokio_tungstenite::tungstenite::Error, Player<S>)>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        match self {
            Self::Startup => {
                (left_player, right_player) =
                    startup::wait_game_0_start(left_player, right_player).await?;
                Ok((
                    Self::Running(RunningState::new(&mut rand::thread_rng())),
                    left_player,
                    right_player,
                ))
            }
            Self::Running(rs) => {
                let game_result =
                    running::run_game_0_loop(&mut left_player.ws, &mut right_player.ws, rs).await;
                Ok((Self::Done(game_result), left_player, right_player))
            }
            Self::Done(d) => Ok((Self::Done(d), left_player, right_player)),
        }
    }
}

/// Current state - or stage - of a game mode 1 game.
pub(super) enum Game1State {
    Startup,
    Running(RunningState),
    Done,
}

impl Game1State {
    /// Create a new game (mode 1) state at the initial stage of startup.
    pub(super) fn new() -> Self {
        Self::Startup
    }

    /// Try to complete the current stage, getting to the next one and returning it.
    pub(super) async fn next_state<S>(
        self,
        mut connection: WebSocketStream<S>,
    ) -> Result<(Self, WebSocketStream<S>), tokio_tungstenite::tungstenite::Error>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        match self {
            Self::Startup => {
                connection = startup::wait_game_1_start(connection).await?;
                Ok((
                    Self::Running(RunningState::new(&mut rand::thread_rng())),
                    connection,
                ))
            }
            Self::Running(rs) => {
                running::run_game_1_loop(&mut connection, rs).await;
                Ok((Self::Done, connection))
            }
            Self::Done => Ok((Self::Done, connection)),
        }
    }
}
