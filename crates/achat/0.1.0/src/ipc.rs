// SPDX-License-Identifier: Apache-2.0

//! Unix-socket IPC layer between the CLI client and the daemon process.

use crate::protocol::{self, IpcRequest, IpcResponse};
use anyhow::{Context, Result};
use std::io;
use std::os::unix::net::UnixStream;
use std::path::Path;

/// Synchronous IPC client: connect to daemon, send request, read response.
pub fn send_request(sock_path: &Path, req: &IpcRequest) -> Result<IpcResponse> {
    let mut stream = UnixStream::connect(sock_path).context("cannot connect to daemon")?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();
    protocol::write_frame(&mut stream, req).context("failed to send request")?;
    protocol::read_frame(&mut stream).context("failed to read response")
}

/// Async IPC server: listen on Unix socket, dispatch requests to handler.
pub async fn run_server(
    sock_path: &Path,
    handler: impl Fn(IpcRequest) -> IpcResponse + Send + Sync + 'static,
) -> io::Result<()> {
    // Remove stale socket
    if sock_path.exists() {
        std::fs::remove_file(sock_path)?;
    }
    let listener = tokio::net::UnixListener::bind(sock_path)?;
    let handler = std::sync::Arc::new(handler);
    loop {
        let (stream, _) = listener.accept().await?;
        let handler = handler.clone();
        tokio::spawn(async move {
            let (mut reader, mut writer) = tokio::io::split(stream);
            if let Ok(req) = protocol::read_frame_async::<IpcRequest>(&mut reader).await {
                let resp = handler(req);
                let _ = protocol::write_frame_async(&mut writer, &resp).await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn ipc_ping_pong() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("test.sock");

        // Spawn server in background
        let sock_clone = sock.clone();
        let server_handle = tokio::spawn(async move {
            let _ = run_server(&sock_clone, |req| match req {
                IpcRequest::Ping => IpcResponse::Ok { id: None },
                IpcRequest::ListAgents
                | IpcRequest::Send { .. }
                | IpcRequest::Inbox { .. }
                | IpcRequest::Feed { .. }
                | IpcRequest::JoinGroup { .. }
                | IpcRequest::LeaveGroup { .. }
                | IpcRequest::Log { .. }
                | IpcRequest::Status
                | IpcRequest::Shutdown
                | IpcRequest::Help => IpcResponse::Error("unexpected".into()),
            })
            .await;
        });

        // Wait for socket to appear
        for _ in 0..50 {
            if sock.exists() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        assert!(sock.exists(), "IPC socket never appeared");

        // Send a Ping request using synchronous client (must run on blocking
        // thread so the async server tasks can progress on the tokio runtime).
        let sock_clone = sock.clone();
        let resp =
            tokio::task::spawn_blocking(move || send_request(&sock_clone, &IpcRequest::Ping))
                .await
                .unwrap()
                .unwrap();
        assert!(
            matches!(resp, IpcResponse::Ok { id: None }),
            "expected Ok {{ id: None }}, got {resp:?}"
        );

        server_handle.abort();
    }
}
