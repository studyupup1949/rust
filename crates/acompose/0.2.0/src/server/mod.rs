pub mod connection;

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use agent_client_protocol::Lines;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{WebSocketStream, accept_hdr_async_with_config};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::compositor::Compositor;
use connection::ClientConnection;

/// Compose WebSocket server — exposes all composed sessions over a single WebSocket.
pub struct Server {
    compositor: Arc<Compositor>,
    bind_address: SocketAddr,
}

impl Server {
    #[must_use]
    pub fn new(compositor: Arc<Compositor>, bind_address: SocketAddr) -> Self {
        Self {
            compositor,
            bind_address,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.bind_address).await?;
        info!(
            "Compose WebSocket server listening on {}",
            self.bind_address
        );
        loop {
            let (stream, addr) = listener.accept().await?;
            let compositor = Arc::clone(&self.compositor);
            let cancel_token = compositor.cancel_token();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, addr, compositor, cancel_token).await {
                    error!(%addr, error = %e, "Compose server connection error");
                }
            });
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
    compositor: Arc<Compositor>,
    cancel_token: CancellationToken,
) -> anyhow::Result<()> {
    let config = WebSocketConfig::default().max_message_size(None);
    let ws = accept_hdr_async_with_config(
        stream,
        #[allow(clippy::result_large_err)]
        |req: &Request, mut response: Response| {
            if let Some(protocol) = req.headers().get("Sec-WebSocket-Protocol")
                && let Ok(protocol) = protocol.to_str()
            {
                let first = protocol
                    .split(',')
                    .next()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("");
                if let Ok(value) = first.parse() {
                    response
                        .headers_mut()
                        .insert("Sec-WebSocket-Protocol", value);
                }
            }
            Ok::<_, ErrorResponse>(response)
        },
        Some(config),
    )
    .await?;
    info!(%addr, "Compose server client connected");

    // Use a child token of the compositor's shutdown token so that proxy
    // connection tasks are cancelled together with the compositor, and so
    // that per-session forward tasks spawned during session/load are cancelled
    // when this connection closes.
    let cancel = cancel_token.child_token();
    let lines = websocket_lines(ws);

    ClientConnection::new(compositor)
        .serve(lines, cancel.clone())
        .await;

    // Stop any per-session forward tasks that are still running.
    cancel.cancel();

    info!(%addr, "Compose server client disconnected");
    Ok(())
}

/// Bridge a WebSocket stream to a line-based ACP transport.
///
/// Each WebSocket text message becomes one JSON-RPC line; outgoing JSON-RPC
/// lines are sent as text messages.
fn websocket_lines<S>(
    ws: WebSocketStream<S>,
) -> Lines<
    impl futures::Sink<String, Error = io::Error> + Send,
    impl futures::Stream<Item = io::Result<String>> + Send,
>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (ws_write, ws_read) = ws.split();

    let outgoing = ws_write
        .sink_map_err(io::Error::other)
        .with(|text: String| {
            futures::future::ready(Ok::<_, io::Error>(Message::Text(text.into())))
        });

    let incoming = ws_read.filter_map(|msg| async move {
        match msg {
            Ok(Message::Text(t)) => Some(Ok(t.to_string())),
            Ok(Message::Close(_) | _) => None,
            Err(e) => Some(Err(io::Error::other(e))),
        }
    });

    Lines::new(outgoing, incoming)
}
