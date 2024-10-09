//! Combining incoming connections and their consumers, ran in spawn threads.

use std::fmt::{Debug, Display};
use std::future::Future;

use nix::sys::socket::{setsockopt, sockopt};
use rand::distributions::{Alphanumeric, DistString};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::WebSocketStream;

/// Number of consecutive accept failures at which it is considered an error.
const MAX_FAILURES: u32 = 3;

/// Dispatcher of incoming connections to asynchronous tokio tasks created on-the-fly.
pub struct OnAcceptGenerator {
    listener: TcpListener,
    consecutive_accept_fail_count: u32,
}

impl OnAcceptGenerator {
    /// Create a new [`OnAcceptGenerator`], which will assign connections incoming on the given [`TcpListener`] to new
    /// tasks.
    pub fn new(tcp_listener: TcpListener) -> OnAcceptGenerator {
        OnAcceptGenerator {
            listener: tcp_listener,
            consecutive_accept_fail_count: 0,
        }
    }

    /// Await for an incoming tcp connection, upgrade it to a websocket connection, and pass it to the given task,
    /// spawned on a [`tokio::task`].
    pub async fn generate_next_task<F, T>(
        &mut self,
        tls_acceptor: &TlsAcceptor,
        task_set: &mut JoinSet<F::Output>,
        task_to_spawn: T,
    ) -> Result<(), ()>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static + Debug,
        T: FnOnce(WebSocketStream<TlsStream<TcpStream>>, String) -> F,
    {
        let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);

        let stream = match tcp_accept_with_opts(&mut self.listener).await {
            Ok(stream) => stream,
            Err(e) => return self.handle_tcp_accept_error(&id, e),
        };

        log::trace!("Accepted a TCP connection with {id}. Trying to upgrade it to Tls...");
        let tls_stream = match tls_acceptor.accept(stream).await {
            Ok(tls_stream) => tls_stream,
            Err(e) => {
                log::warn!("{id}: Failed to upgrade a connection to Tls : {e}.");
                return Ok(());
            }
        };

        log::trace!("Accepted a Tls connection with {id}. Trying to upgrade it to WSS...");
        let websocket = match ws_accept(tls_stream).await {
            Ok(websocket) => websocket,
            Err(e) => {
                log::info!("Failed to upgrade the Tls connection to websocket with error : {e}.");
                return Ok(());
            }
        };

        log::info!("Established a websocket connection with {id}. Spawning a task to handle it.");
        task_set.spawn(task_to_spawn(websocket, id));
        self.consecutive_accept_fail_count = 0;

        Ok(())
    }

    /// Log the error received, and return [`Err`] if [`Self`] has encountered [`MAX_FAILURES`] consecutive errors.
    fn handle_tcp_accept_error<D: Display>(&mut self, id: &D, e: std::io::Error) -> Result<(), ()> {
        self.consecutive_accept_fail_count += 1;
        if self.consecutive_accept_fail_count != MAX_FAILURES {
            log::warn!(
                "{id}: Accepting an incoming connection failed [{}/{MAX_FAILURES}] with error : {e}.",
                self.consecutive_accept_fail_count
            );
            Ok(())
        } else {
            log::error!(
                "{id}: Accepting an incoming connection failed [{}/{MAX_FAILURES}] with error : {e}. \
                        Threshold hit, considering this an error.",
                self.consecutive_accept_fail_count
            );
            Err(())
        }
    }
}

/// Accept a tcp connection from the listener, and set the socket to no delay to disable Nagle's algorithm.
async fn tcp_accept_with_opts(listener: &mut TcpListener) -> std::io::Result<TcpStream> {
    let (stream, _) = listener.accept().await?;
    setsockopt(&stream, sockopt::TcpNoDelay, &true)?;
    Ok(stream)
}

/// Upgrade the stream connection to a WebSocket connection. The configuration supplied sets small buffers.
async fn ws_accept<S>(
    tcp_stream: S,
) -> Result<WebSocketStream<S>, tokio_tungstenite::tungstenite::Error>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    const NO_BUFFER: usize = 0;
    const KB_1: usize = 1 << 10;
    const KB_2: usize = 2 << 10;
    let ws_config = WebSocketConfig {
        write_buffer_size: NO_BUFFER,
        max_write_buffer_size: KB_1,
        max_message_size: Some(KB_2),
        max_frame_size: Some(KB_1),
        ..Default::default()
    };
    tokio_tungstenite::accept_async_with_config(tcp_stream, Some(ws_config)).await
}
