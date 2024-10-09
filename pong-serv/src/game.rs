//! Implementation of the logic of the Pong game.
//!
//! This mod defines and exposes the entrypoint function [`play_game_mode_0`], implemented below in sub-mods and in the
//! [`crate::protocol`] mod.

use std::sync::Arc;
use std::time::SystemTime;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_postgres::types::ToSql;
use tokio_tungstenite::{tungstenite, WebSocketStream};

pub use side::Side;
use state::Game0State;

use crate::game::state::{Game1State, GameResult};

mod combined_send;
mod engine;
mod side;
mod state;

/// Opaque structure coming out of matchmaking for use by this mod. Represents a player.
#[derive(Debug)]
pub struct Player<S> {
    ws: WebSocketStream<S>,
    id: String,
}

impl<S> Player<S> {
    /// Creates a new [`Player`].
    pub fn new(websocket: WebSocketStream<S>, id: String) -> Player<S> {
        Player { ws: websocket, id }
    }
}

/// Errors encountered while playing the game.
#[derive(thiserror::Error, Debug)]
pub enum PlayingError<S> {
    /// This error happens if a poll to a [`WebSocketStream`] returns an error when sending the
    /// [`protocol::GameStartMessage`] to a client.
    #[error("an error at the websocket layer occurred during pre-game grace period : {0}")]
    ClientError(tungstenite::Error, (WebSocketStream<S>, String)),

    /// This error happens when an interaction with the database fails. This should never happen if everything is
    /// configured correctly, and therefore indicates a runtime issue outside the scope of this program.
    #[error("an error with the database occurred : {0}")]
    DatabaseError(#[from] tokio_postgres::Error),
}

impl<S> From<(tungstenite::Error, Player<S>)> for PlayingError<S> {
    fn from((error, player): (tungstenite::Error, Player<S>)) -> Self {
        PlayingError::ClientError(error, (player.ws, player.id))
    }
}

/// Play out a game of Pong opposing the two [`Player`]s. Returns them for further playing if no error occurred. The
/// only possible errors are websocket-related and database-related.
pub async fn play_game_mode_0<S>(
    mut left_player: Player<S>,
    mut right_player: Player<S>,
    db_client: &Arc<tokio_postgres::Client>,
) -> Result<(Player<S>, Player<S>), PlayingError<S>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let game_start_time_point = SystemTime::now();

    let mut game_state = Game0State::new();
    let (game_result, pl, pr) = loop {
        (game_state, left_player, right_player) =
            match game_state.next_state(left_player, right_player).await? {
                (Game0State::Done(result), pl, pr) => break (result, pl, pr),
                other_state => other_state,
            }
    };
    (left_player, right_player) = (pl, pr);

    let game_end_time_point = SystemTime::now();
    write_game_result_to_database(
        db_client,
        &left_player.id,
        &right_player.id,
        game_start_time_point,
        game_end_time_point,
        game_result,
    )
    .await?;

    Ok((left_player, right_player))
}

pub async fn play_game_mode_1<S>(
    mut connection: WebSocketStream<S>,
) -> Result<WebSocketStream<S>, tokio_tungstenite::tungstenite::Error>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut game_state = Game1State::new();
    let connection = loop {
        (game_state, connection) = match game_state.next_state(connection).await? {
            (Game1State::Done, connection) => break connection,
            other_state => other_state,
        };
    };
    Ok(connection)
}

/// Try to write the game outcome to the database. Errors here are database errors - this is hard.
async fn write_game_result_to_database(
    db_client: &Arc<tokio_postgres::Client>,
    pl_id: &str,
    pr_id: &str,
    game_start_time_point: SystemTime,
    game_end_time_point: SystemTime,
    game_result: GameResult,
) -> Result<(), tokio_postgres::Error> {
    let query = match game_result.winner {
        Side::Left => "with id1 as (select id from account_player where username = $1), \
                            id2 as (select id from account_player where username = $2) \
                       insert \
                       into account_gameresult(p1_score, p2_score, date, duration, p1_id, p2_id, winner_id) \
                       values($3, $4, $5, cast ($6 as timestamp with time zone) - $5, \
                              (select id from id1), (select id from id2), (select id from id1));",
        Side::Right => "with id1 as (select id from account_player where username = $1), \
                             id2 as (select id from account_player where username = $2) \
                        insert \
                        into account_gameresult(p1_score, p2_score, date, duration, p1_id, p2_id, winner_id) \
                        values($3, $4, $5, cast ($6 as timestamp with time zone) - $5, \
                               (select id from id1), (select id from id2), (select id from id2));",
    };
    let parameters: [&(dyn ToSql + Sync); 6] = [
        &pl_id,
        &pr_id,
        &i16::try_from(game_result.score[0]).expect("Score is beyond an i16."),
        &i16::try_from(game_result.score[1]).expect("Score is beyond an i16."),
        &game_start_time_point,
        &game_end_time_point,
    ];
    db_client.execute(query, &parameters).await.map(|_| ())
}
