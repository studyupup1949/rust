// SPDX-License-Identifier: Apache-2.0

//! TCP transport layer for agent-to-agent message delivery.
//!
//! Provides a listener that accepts inbound messages and a send function
//! for outbound delivery. Returns (`listener_port`, `join_handle`).

use crate::protocol::{self, Message};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

/// Start a TCP listener that forwards received messages into the given channel.
///
/// Returns the bound port and a join handle for the listener task.
pub async fn start_listener(
    tx: mpsc::Sender<Message>,
) -> io::Result<(u16, tokio::task::JoinHandle<()>)> {
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();

    let handle = tokio::spawn(async move {
        while let Ok((stream, _addr)) = listener.accept().await {
            let tx = tx.clone();
            tokio::spawn(async move {
                let (mut reader, mut writer) = tokio::io::split(stream);
                if let Ok(msg) = protocol::read_frame_async::<Message>(&mut reader).await {
                    // Send ACK
                    let _ = tokio::io::AsyncWriteExt::write_all(&mut writer, &[0x06]).await;
                    let _ = tx.send(msg).await;
                }
            });
        }
    });

    Ok((port, handle))
}

/// Send a message to a remote agent via TCP.
pub async fn send_message(addr: SocketAddr, msg: &Message) -> io::Result<()> {
    let stream = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::net::TcpStream::connect(addr),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "connect timeout"))??;

    let (mut reader, mut writer) = tokio::io::split(stream);
    protocol::write_frame_async(&mut writer, msg).await?;

    // Wait for ACK
    let mut ack = [0u8; 1];
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::io::AsyncReadExt::read_exact(&mut reader, &mut ack),
    )
    .await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Target;

    fn make_test_msg() -> Message {
        Message {
            id: "transport-test".into(),
            from: "alice".into(),
            to: Target::Direct("bob".into()),
            content: "hello transport".into(),
            ts: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[tokio::test]
    async fn tcp_send_receive_roundtrip() {
        let (tx, mut rx) = mpsc::channel::<Message>(16);
        let (port, _handle) = start_listener(tx).await.unwrap();
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

        let msg = make_test_msg();
        send_message(addr, &msg).await.unwrap();

        let received = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
            .await
            .expect("timed out waiting for message")
            .expect("channel closed");

        assert_eq!(received.id, "transport-test");
        assert_eq!(received.from, "alice");
        assert_eq!(received.content, "hello transport");
    }

    #[tokio::test]
    async fn send_to_bad_addr_fails() {
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let msg = make_test_msg();
        let result = send_message(addr, &msg).await;
        assert!(result.is_err());
    }
}
