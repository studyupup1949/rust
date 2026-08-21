// SPDX-License-Identifier: Apache-2.0

//! Wire protocol types and length-prefixed frame codec for agent-to-agent
//! and CLI-to-daemon communication.

use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

// --- Wire message (agent-to-agent over TCP) ---

/// A message exchanged between agents over TCP.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use = "received messages should be stored or forwarded"]
pub struct Message {
    /// Unique message identifier.
    pub id: String,
    /// Name of the sending agent.
    pub from: String,
    /// Routing target for this message.
    pub to: Target,
    /// Message body.
    pub content: String,
    /// RFC 3339 timestamp.
    pub ts: String,
}

/// Routing target for a message.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Target {
    /// Addressed to a single agent by name.
    Direct(String),
    /// Addressed to all members of a named group.
    Group(String),
    /// Addressed to every reachable agent.
    Broadcast,
}

// --- Agent info (from mDNS discovery) ---

/// Metadata about a discovered peer agent.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentInfo {
    /// Agent name.
    pub name: String,
    /// IP address.
    pub addr: String,
    /// TCP port.
    pub port: u16,
    /// Groups this agent belongs to.
    pub groups: Vec<String>,
}

impl AgentInfo {
    /// Parse `addr:port` into a `SocketAddr`.
    pub fn socket_addr(&self) -> Option<std::net::SocketAddr> {
        format!("{}:{}", self.addr, self.port).parse().ok()
    }
}

// --- IPC protocol (CLI <-> daemon over Unix socket) ---

/// Request sent from the CLI to the daemon over a Unix socket.
#[derive(Serialize, Deserialize, Debug)]
pub enum IpcRequest {
    /// Health check.
    Ping,
    /// List all discovered agents.
    ListAgents,
    /// Send a message to a specific target (direct, group, or broadcast).
    Send { to: Target, content: String },
    /// Retrieve direct messages.
    Inbox { limit: usize },
    /// Retrieve broadcast messages.
    Feed { limit: usize },
    /// Join a named group.
    JoinGroup { group: String },
    /// Leave a named group.
    LeaveGroup { group: String },
    /// Retrieve message history, optionally filtered by target.
    Log {
        target: Option<String>,
        limit: usize,
    },
    /// Query daemon status.
    Status,
    /// Request graceful shutdown.
    Shutdown,
    /// List available commands.
    Help,
}

/// Response sent from the daemon back to the CLI.
#[derive(Serialize, Deserialize, Debug)]
#[must_use = "IPC responses should be checked for errors"]
pub enum IpcResponse {
    /// Generic success, optionally carrying a message id.
    Ok { id: Option<String> },
    /// List of discovered agents.
    Agents(Vec<AgentInfo>),
    /// A batch of messages with pagination metadata.
    Messages {
        msgs: Vec<Message>,
        total: usize,
        truncated: bool,
    },
    /// Daemon status snapshot.
    Status {
        name: String,
        uptime_secs: u64,
        groups: Vec<String>,
        peers: usize,
    },
    /// Available command descriptions.
    Help { commands: Vec<CommandInfo> },
    /// An error message.
    Error(String),
}

/// Description of a single CLI command.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandInfo {
    /// Command name.
    pub name: String,
    /// Argument synopsis.
    pub args: String,
    /// Human-readable description.
    pub desc: String,
}

// --- Length-prefixed frame codec ---
// Frame: [4 bytes big-endian length][JSON bytes]

const MAX_FRAME_LEN: usize = 4 * 1024 * 1024; // 4 MB

/// Serialize a value to a length-prefixed JSON frame (bytes).
fn encode_frame<T: Serialize>(value: &T) -> io::Result<Vec<u8>> {
    let json =
        serde_json::to_vec(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let len = u32::try_from(json.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "frame exceeds u32 length"))?;
    let mut buf = Vec::with_capacity(4 + json.len());
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(&json);
    Ok(buf)
}

/// Validate frame length header and deserialize the payload.
fn decode_frame<T: for<'de> Deserialize<'de>>(len_buf: [u8; 4], payload: &[u8]) -> io::Result<T> {
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large (>4MB)",
        ));
    }
    serde_json::from_slice(payload).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Write a length-prefixed JSON frame to a synchronous writer.
pub fn write_frame<W: Write, T: Serialize>(writer: &mut W, value: &T) -> io::Result<()> {
    let buf = encode_frame(value)?;
    writer.write_all(&buf)?;
    writer.flush()
}

/// Read a length-prefixed JSON frame from a synchronous reader.
pub fn read_frame<R: Read, T: for<'de> Deserialize<'de>>(reader: &mut R) -> io::Result<T> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large (>4MB)",
        ));
    }
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;
    decode_frame(len_buf, &payload)
}

/// Write a length-prefixed JSON frame to an async writer.
pub async fn write_frame_async<T: Serialize>(
    writer: &mut (impl tokio::io::AsyncWriteExt + Unpin),
    value: &T,
) -> io::Result<()> {
    let buf = encode_frame(value)?;
    writer.write_all(&buf).await?;
    writer.flush().await
}

/// Read a length-prefixed JSON frame from an async reader.
pub async fn read_frame_async<T: for<'de> Deserialize<'de>>(
    reader: &mut (impl tokio::io::AsyncReadExt + Unpin),
) -> io::Result<T> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large (>4MB)",
        ));
    }
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    decode_frame(len_buf, &payload)
}

// --- Helpers ---

/// Return the list of available CLI commands with their argument synopses.
pub fn command_list() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "up".into(),
            args: "<name>".into(),
            desc: "Start daemon and register on LAN".into(),
        },
        CommandInfo {
            name: "down".into(),
            args: String::new(),
            desc: "Stop daemon".into(),
        },
        CommandInfo {
            name: "attach".into(),
            args: "<name>".into(),
            desc: "Rebind identity to a running daemon".into(),
        },
        CommandInfo {
            name: "send".into(),
            args: "<target> [message]".into(),
            desc: "Send to @agent or group. Reads stdin if no message arg".into(),
        },
        CommandInfo {
            name: "cast".into(),
            args: "[message]".into(),
            desc: "Broadcast to all. Reads stdin if no message arg".into(),
        },
        CommandInfo {
            name: "inbox".into(),
            args: "[-n N]".into(),
            desc: "Show direct messages (default 20)".into(),
        },
        CommandInfo {
            name: "feed".into(),
            args: "[-n N]".into(),
            desc: "Show broadcasts (default 20)".into(),
        },
        CommandInfo {
            name: "join".into(),
            args: "<group>".into(),
            desc: "Join a named group".into(),
        },
        CommandInfo {
            name: "leave".into(),
            args: "<group>".into(),
            desc: "Leave a group".into(),
        },
        CommandInfo {
            name: "log".into(),
            args: "[@agent|group] [-n N]".into(),
            desc: "Message history".into(),
        },
        CommandInfo {
            name: "ls".into(),
            args: String::new(),
            desc: "List online agents".into(),
        },
        CommandInfo {
            name: "status".into(),
            args: String::new(),
            desc: "Daemon status".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn sample_msg(id: &str, content: &str) -> Message {
        Message {
            id: id.into(),
            from: "alice".into(),
            to: Target::Direct("bob".into()),
            content: content.into(),
            ts: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn frame_roundtrip_message() {
        let msg = sample_msg("test-id", "hello");
        let mut buf = Vec::new();
        write_frame(&mut buf, &msg).unwrap();
        let decoded: Message = read_frame(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded.id, "test-id");
        assert_eq!(decoded.content, "hello");
    }

    #[test]
    fn frame_roundtrip_ipc_request() {
        let req = IpcRequest::Send {
            to: Target::Group("backend".into()),
            content: "task done".into(),
        };
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        let decoded: IpcRequest = read_frame(&mut Cursor::new(&buf)).unwrap();
        assert!(matches!(decoded, IpcRequest::Send { .. }));
    }

    #[test]
    fn frame_rejects_oversized() {
        let header = (5_000_001u32).to_be_bytes();
        let result = read_frame::<_, Message>(&mut Cursor::new(&header));
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn agent_info_socket_addr() {
        let info = AgentInfo {
            name: "test".into(),
            addr: "127.0.0.1".into(),
            port: 8080,
            groups: vec![],
        };
        assert_eq!(info.socket_addr().unwrap().port(), 8080);
    }

    #[test]
    fn agent_info_bad_addr_returns_none() {
        let info = AgentInfo {
            name: "test".into(),
            addr: "not-an-ip".into(),
            port: 8080,
            groups: vec![],
        };
        assert!(info.socket_addr().is_none());
    }

    #[test]
    fn target_serde_roundtrip() {
        for target in [
            Target::Direct("alice".into()),
            Target::Group("backend".into()),
            Target::Broadcast,
        ] {
            let json = serde_json::to_string(&target).unwrap();
            let decoded: Target = serde_json::from_str(&json).unwrap();
            assert_eq!(serde_json::to_string(&decoded).unwrap(), json);
        }
    }

    #[tokio::test]
    async fn async_frame_roundtrip() {
        let msg = sample_msg("async-id", "async hello");
        let mut buf = Vec::new();
        write_frame_async(&mut buf, &msg).await.unwrap();
        let mut reader = &buf[..];
        let decoded: Message = read_frame_async(&mut reader).await.unwrap();
        assert_eq!(decoded.id, "async-id");
        assert_eq!(decoded.content, "async hello");
    }

    #[tokio::test]
    async fn async_frame_rejects_oversized() {
        let header = (5_000_001u32).to_be_bytes();
        let mut reader = &header[..];
        let result = read_frame_async::<Message>(&mut reader).await;
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn command_list_completeness() {
        let cmds = command_list();
        let names: Vec<&str> = cmds.iter().map(|c| c.name.as_str()).collect();
        for expected in ["up", "send", "inbox", "status"] {
            assert!(names.contains(&expected), "missing command: {expected}");
        }
        for cmd in &cmds {
            assert!(
                !cmd.desc.is_empty(),
                "empty description for command: {}",
                cmd.name
            );
        }
    }
}
