//! Implementation of the client-server pong communication protocol
//!
//! This module provides structures mapping the protocol messages, helper functions for messages and an entrypoint
//! function that runs the protocol on a given [`WebSocketStream`] connection : [`execute_protocol_on_connection`].
//!
//! The structures are :
//! * Serializable : [`GameCompletedMessage`], [`PointScoredMessage`] and [`PositionUpdateMessage`] wrapped in the enum
//!   [`ServerToClientMessage`], and [`GameMode0StartMessage`].
//! * Deserializable : [`HelloMessage`].
//!
//! The messages received from the client are processed through the helper functions [`parse_gm0_input_message`] and
//! [`receive_hello_message`].

use std::fmt::Display;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

pub use messages::game_running::{
    parse_gm0_input_message, parse_gm1_input_message, GameAbortedMessage, GameCompletedMessage,
    PointScoredMessage, PositionUpdateMessage, ServerToClientMessage,
};
pub use messages::game_start::{GameMode0StartMessage, GameMode1StartMessage};
use messages::hello::GameModes;
use messages::hello::{receive_hello_message, HelloMessage};

use crate::game::{play_game_mode_0, play_game_mode_1, PlayingError};
use crate::match_making;

pub mod constants;
mod messages;
mod side;

/// The current maximum version of the protocol supported.
const SUPPORTED_PROTO_VERSION: u8 = 3;

/// Receives a [`HelloMessage`], and runs the combination of match-making and game type requested.
pub async fn execute_protocol_on_connection<S, D>(
    mut websocket: WebSocketStream<S>,
    log_id: D,
    match_maker: Arc<match_making::MatchMaker<S>>,
    db_client: Arc<tokio_postgres::Client>,
) where
    S: AsyncRead + AsyncWrite + Unpin,
    D: Display,
{
    log::info!("{log_id}: Beginning to unroll the protocol with a client.");
    match receive_hello_message(&mut websocket).await {
        Ok(hello_message) => {
            if is_id_valid(&db_client, &hello_message.id).await {
                dispatch_requested_game_mode(
                    websocket,
                    &log_id,
                    match_maker,
                    db_client,
                    hello_message,
                )
                .await;
            } else {
                log::info!(
                    "{log_id}: The client sent an id that doesn't exist in the users database."
                )
            }
        }
        Err(e) => log::info!("{log_id}: Error while receiving a hello message : {e}."),
    }
    log::info!("{log_id}: Protocol done.");
}

/// Check in the database if the id given by the remote client exists.
async fn is_id_valid(db_client: &Arc<tokio_postgres::Client>, id: &str) -> bool {
    match db_client
        .query_one("select * from account_player where username = $1", &[&id])
        .await
    {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Try to launch the requested game mode, if the requested protocol version is supported and if the game exists for
/// this version.
async fn dispatch_requested_game_mode<S, D>(
    websocket: WebSocketStream<S>,
    log_id: &D,
    match_maker: Arc<match_making::MatchMaker<S>>,
    db_client: Arc<tokio_postgres::Client>,
    hello_message: HelloMessage,
) where
    S: AsyncRead + AsyncWrite + Unpin,
    D: Display,
{
    match hello_message {
        HelloMessage { proto_version, .. } if proto_version != SUPPORTED_PROTO_VERSION => {
            log::info!("{log_id}: Received a request for protocol version {proto_version}, but is not supported.");
        }
        HelloMessage { id, game_mode, .. } if game_mode == GameModes::MatchMadeRemote1v1.into() => {
            launch_game_mode_0(websocket, id, &match_maker, &db_client, log_id).await;
        }
        HelloMessage { game_mode, .. } if game_mode == GameModes::Local1v1.into() => {
            launch_game_mode_1(websocket, log_id).await;
        }
        HelloMessage { game_mode: gm, .. } => {
            log::info!(
                "{log_id}: Received a request for game mode {gm}, but requested protocol version \
                {SUPPORTED_PROTO_VERSION} does not support it."
            );
        }
    }
}

/// Answer to a game mode 0 request : join this task's connection with another task's connections chosen by the
/// [`match_making::MatchMaker`], then make them play together. Failed startups lead to a return of a player to the
/// match making queue.
async fn launch_game_mode_0<S, D>(
    mut websocket: WebSocketStream<S>,
    mut id: String,
    match_maker: &Arc<match_making::MatchMaker<S>>,
    db_client: &Arc<tokio_postgres::Client>,
    log_id: &D,
) where
    S: AsyncRead + AsyncWrite + Unpin,
    D: Display,
{
    log::trace!("{log_id}: [Version {SUPPORTED_PROTO_VERSION}]-[Game mode 0] request received.");
    'new_match_making_attempt: loop {
        match match_making::join_opponents(websocket, id, &match_maker, log_id).await {
            Some((pl, pr)) => {
                log::trace!("{log_id}: Two connections have been joined. Playing a game.");
                match play_game_mode_0(pl, pr, db_client).await {
                    Ok(_) => log::trace!("{log_id}: The game has been played to completion."),
                    Err(PlayingError::ClientError(e, (new_websocket, new_id))) => {
                        log::info!("{log_id}: Game startup failed : {e}.");
                        (websocket, id) = (new_websocket, new_id);
                        continue 'new_match_making_attempt;
                    }
                    Err(PlayingError::DatabaseError(e)) => {
                        log::error!("{log_id}: Database error during game : {e}.");
                    }
                }
            }
            None => {
                log::info!("{log_id}: Connection has been given away to another task.");
            }
        }
        break;
    }
}

/// Answer to a game mode 1 request.
async fn launch_game_mode_1<S, D>(websocket: WebSocketStream<S>, log_id: &D)
where
    S: AsyncRead + AsyncWrite + Unpin,
    D: Display,
{
    log::trace!("{log_id}: [Version {SUPPORTED_PROTO_VERSION}]-[Game mode 1] request received.");
    log::trace!("{log_id}: Playing the requested game.");
    match play_game_mode_1(websocket).await {
        Ok(_) => log::trace!("{log_id} The game has been played to completion."),
        Err(e) => log::info!("{log_id} Error encountered while playing the game : {e}."),
    }
}
