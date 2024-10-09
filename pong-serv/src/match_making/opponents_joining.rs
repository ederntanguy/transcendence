//! Joining of two players/connections to a same task.
//!
//! This mod contains the logic for a task to send its connection to another task, or be sent another task's
//! connection, depending on who contacted the matchmaker first. This is implemented in [`join_opponents`].

use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::oneshot;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::WebSocketStream;

use crate::game::Player;
use crate::match_making::MatchMaker;

/// Determine whether another task is waiting or not, then either send or receive the [`WebSocketStream`] and identity.
///
/// This function does the minimal amount of blocking work with the [`MatchMaker`] to learn about its role and get the
/// [`oneshot`] channel needed. It then fulfills its role :
/// * If it is first, it waits for the other's [`GiverToExecutorData`]. From the caller's point of view, it returns both
///   [`WebSocketStream`]s. In reality, if this task's [`WebSocketStream`] closes while waiting for the other task's, it
///   will wait for another task to send itself, and start over as if doing a re-entry in the function.
/// * If it is second, it sends its own [`GiverToExecutorData`], and returns nothing.
pub async fn join_opponents<S, D>(
    mut websocket: WebSocketStream<S>,
    mut id: String,
    match_maker: &Arc<MatchMaker<S>>,
    log_id: &D,
) -> Option<(Player<S>, Player<S>)>
where
    D: Display,
    S: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        return match GameRunRole::extract_from_mutex(&match_maker.mutex) {
            GameRunRole::Giver(giver_to_executor_sender) => {
                log::trace!("{log_id}: GameRunRole is Giver.");
                let data = GiverToExecutorData {
                    giver_websocket: websocket,
                    giver_id: id,
                };
                //Sending cannot fail, as we make sure the receiving end is not dropped.
                giver_to_executor_sender.send(data).map_err(|_| ()).unwrap();
                None
            }
            GameRunRole::Executor(mut giver_to_executor_receiver) => {
                log::trace!("{log_id}: GameRunRole is Executor. Waiting for Giver data.");
                let GiverToExecutorData {
                    giver_websocket,
                    giver_id,
                } = match wait_for_giver_data(&mut giver_to_executor_receiver, &mut websocket).await
                {
                    Ok(data) => data,
                    Err(e) => {
                        log::info!("{log_id}: Disconnection detected");
                        log::debug!("{log_id}: Disconnection cause : {e} | {e:?}.");
                        log::info!("{log_id}: Waiting for a Giver to take over this task thread.");
                        GiverToExecutorData {
                            giver_websocket: websocket,
                            giver_id: id,
                        } = loop {
                            //Only spurious errors can happen, as we make sure the sending end is not dropped.
                            if let Ok(data) = (&mut giver_to_executor_receiver).await {
                                break data;
                            }
                        };
                        log::info!(
                            "{log_id}: The task thread has been taken over by a new connection."
                        );
                        continue;
                    }
                };
                log::trace!("{log_id}: Giver data received.");
                Some((
                    Player::new(giver_websocket, giver_id),
                    Player::new(websocket, id),
                ))
            }
        };
    }
}

/// Possible errors when waiting for Giver data.
#[derive(thiserror::Error, Debug)]
pub enum WaitError {
    /// This error happens when a poll to a [`WebSocketStream`] returns an error.
    #[error("Connection error or close while waiting for someone to join : {0}")]
    ConnectionError(#[from] Error),

    /// This error happens when a poll to a [`WebSocketStream`] returns [`None`], or that the connection has been
    /// closed.
    #[error("Connection lost while waiting for someone to join : closed or lost")]
    ConnectionLost,

    /// This error is given when the client sends any data other than a ping, or a pong if waiting for someone to
    /// join. The protocol is enforced strictly.
    #[error("Received an unexpected message from the client : {0}")]
    ProtocolViolation(Message),
}

/// Time between each keep-alive ping sent to the client.
const PING_INTERVAL: u64 = 15;
/// Maximum time a client has to answer a ping before being considered non-responsive.
const PONG_TIMEOUT: u64 = 5;
/// The ping payload - `b2sum(wait_for_giver_data)`.
const PING_PAYLOAD: [u8; 8] = [1, 3, 0, 7, 3, 15, 3, 4];

/// Wait for the [`GiverToExecutorData`] to be received.
///
/// In the meantime, pings are answered and some pongs are sent on a regular basis. If the client disconnects or doesn't
/// answer pings, return a [`WaitError`].
async fn wait_for_giver_data<S>(
    giver_to_executor_receiver: &mut oneshot::Receiver<GiverToExecutorData<S>>,
    executor_websocket: &mut WebSocketStream<S>,
) -> Result<GiverToExecutorData<S>, WaitError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut ping_interval = tokio::time::interval_at(
        Instant::now() + Duration::from_secs(PING_INTERVAL),
        Duration::from_secs(PING_INTERVAL),
    );
    let mut waiting_for_pong = false;
    let mut pong_timeout = Instant::now();
    loop {
        tokio::select! {
            receive_result = &mut *giver_to_executor_receiver => {
                //All errors should be spurious and fine to ignore, as the program logic doesn't allow the sending end
                //to close before the receiving end.
                if let Ok(data) = receive_result {
                    return Ok(data);
                }
            },
            _ = ping_interval.tick() => {
                //Time to send a ping.
                executor_websocket.send(Message::Ping(Vec::from(PING_PAYLOAD))).await?;
                waiting_for_pong = true;
                pong_timeout = Instant::now() + Duration::from_secs(PONG_TIMEOUT);
            },
            _ = tokio::time::sleep_until(pong_timeout), if waiting_for_pong => {
                //No ping received in time.
                return Err(WaitError::ConnectionLost);
            },
            msg = executor_websocket.next() => {
                //Handle message, potentially clearing the ping timeout if we were waiting on one.
                waiting_for_pong = handle_websocket_event(msg, waiting_for_pong)?;
            },
        }
    }
}

/// Handle a message received on the given websocket, expecting
/// * a ping
/// * a pong if `waiting_for_pong` is `true`
///
/// and returning an error in the other cases, which are
/// * nothing received or reception error
/// * any message type other than [`Message::Ping`] and optionally [`Message::Pong`]
///
/// The [`Ok`] return value will contain whether to keep waiting for a pong : if `waiting_for_pong` was `false` it will
/// be false, and if the latter was `true` it will be turned to `false` if the expected pong is received.
fn handle_websocket_event(
    message: Option<Result<Message, Error>>,
    waiting_for_pong: bool,
) -> Result<bool, WaitError> {
    match message {
        //Send back a pong and don't interfere with the pong we're waiting for
        Some(Ok(Message::Ping(_))) => Ok(waiting_for_pong),
        //Handle a pong, checking that it is wanted and correct
        Some(Ok(Message::Pong(p))) => {
            if waiting_for_pong && p == PING_PAYLOAD {
                Ok(false)
            } else {
                Err(WaitError::ProtocolViolation(Message::Pong(p)))
            }
        }
        //Received a message we're not expecting.
        Some(Ok(message)) => Err(WaitError::ProtocolViolation(message)),
        //Connection is closed, failed or anything like that.
        None | Some(Err(Error::ConnectionClosed)) => Err(WaitError::ConnectionLost),
        Some(Err(e)) => Err(WaitError::ConnectionError(e)),
    }
}

/// Connection and identity sent from the second task contacting the matchmaker to one already waiting.
pub(super) struct GiverToExecutorData<S> {
    giver_websocket: WebSocketStream<S>,
    giver_id: String,
}

/// The role of this task regarding who sends and who receives the other task's connection.
enum GameRunRole<S> {
    /// This task contacted the matchmaker first, and waits for another one to send its [`GiverToExecutorData`]. It will
    /// execute whatever they want to do together.
    Executor(oneshot::Receiver<GiverToExecutorData<S>>),
    /// This task contacted the matchmaker second. It will send its data through a [`oneshot`] channel.
    Giver(oneshot::Sender<GiverToExecutorData<S>>),
}

impl<S> GameRunRole<S> {
    /// * If the [`Mutex`] contains a [`oneshot::Sender`], consume it and return as a [`Self::Giver`].
    /// * If the [`Mutex`] is empty, create a [`oneshot::channel`], and return as a [`Self::Executor`].
    fn extract_from_mutex(
        mutex: &Mutex<Option<oneshot::Sender<GiverToExecutorData<S>>>>,
    ) -> GameRunRole<S> {
        // The lock cannot panic as nothing in the guard's scope can panic.
        let mut maybe_giver_data_sender = mutex.lock().unwrap();
        if maybe_giver_data_sender.is_some() {
            //The take cannot be None and panic, as we just checked that the option is Some.
            let giver_to_executor_sender = maybe_giver_data_sender.take().unwrap();
            GameRunRole::Giver(giver_to_executor_sender)
        } else {
            let (giver_to_executor_sender, giver_to_executor_receiver) = oneshot::channel();
            *maybe_giver_data_sender = Some(giver_to_executor_sender);
            GameRunRole::Executor(giver_to_executor_receiver)
        }
    }
}
