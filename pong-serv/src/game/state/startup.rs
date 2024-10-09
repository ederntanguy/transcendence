use std::time::{Duration, SystemTime};

use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::game::{Player, Side};
use crate::protocol::{GameMode0StartMessage, GameMode1StartMessage};

/// Send the game start message and wait until the deadline sent to clients elapses or a client disconnects.
pub(super) async fn wait_game_0_start<S>(
    mut left_player: Player<S>,
    mut right_player: Player<S>,
) -> Result<(Player<S>, Player<S>), (tokio_tungstenite::tungstenite::Error, Player<S>)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let game_start_time = SystemTime::now() + Duration::from_secs(5);
    let game_start_instant = Instant::now() + Duration::from_secs(5);
    (left_player, right_player) =
        send_mode_0_start_messages(left_player, right_player, game_start_time).await?;
    (left_player, right_player) =
        wait_grace_period(left_player, right_player, game_start_instant).await?;
    Ok((left_player, right_player))
}

async fn send_mode_0_start_messages<S>(
    mut left_player: Player<S>,
    mut right_player: Player<S>,
    game_start_time: SystemTime,
) -> Result<(Player<S>, Player<S>), (tokio_tungstenite::tungstenite::Error, Player<S>)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if let Err(e) = left_player
        .ws
        .send(Message::Binary(
            GameMode0StartMessage::new(&right_player.id, Side::Left, game_start_time).into(),
        ))
        .await
    {
        return Err((e, right_player));
    }
    if let Err(e) = right_player
        .ws
        .send(Message::Binary(
            GameMode0StartMessage::new(&left_player.id, Side::Right, game_start_time).into(),
        ))
        .await
    {
        return Err((e, left_player));
    }
    Ok((left_player, right_player))
}

async fn wait_grace_period<S>(
    mut left_player: Player<S>,
    mut right_player: Player<S>,
    game_start_instant: Instant,
) -> Result<(Player<S>, Player<S>), (tokio_tungstenite::tungstenite::Error, Player<S>)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        tokio::select! {
            msg = right_player.ws.next() => match msg {
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err((e, left_player)),
                None => return Err((tokio_tungstenite::tungstenite::Error::ConnectionClosed, left_player)),
            },
            msg = left_player.ws.next() => match msg {
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err((e, right_player)),
                None => return Err((tokio_tungstenite::tungstenite::Error::ConnectionClosed, right_player)),
            },
            _ = tokio::time::sleep_until(game_start_instant) => return Ok((left_player, right_player)),
        }
    }
}

pub(super) async fn wait_game_1_start<S>(
    mut connection: WebSocketStream<S>,
) -> Result<WebSocketStream<S>, tokio_tungstenite::tungstenite::Error>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let game_start_time = SystemTime::now() + Duration::from_secs(5);
    let game_start_instant = Instant::now() + Duration::from_secs(5);
    connection
        .send(Message::Binary(
            GameMode1StartMessage::new(game_start_time).into(),
        ))
        .await?;
    tokio::time::sleep_until(game_start_instant).await;
    Ok(connection)
}
