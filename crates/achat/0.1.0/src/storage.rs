// SPDX-License-Identifier: Apache-2.0

//! Persistent message storage backed by JSONL files.
//!
//! Each agent gets a directory under `~/.achat/agents/<name>/messages/` where
//! conversations are stored as one-file-per-target JSONL logs.

use crate::protocol::Message;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// Return the base directory. Respects `ACHAT_HOME` for testing and custom
/// deployments; defaults to `~/.achat`.
pub fn base_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("ACHAT_HOME") {
        return PathBuf::from(custom);
    }
    std::env::var("HOME")
        .map_or_else(|_| PathBuf::from("."), PathBuf::from)
        .join(".achat")
}

/// Return the agent-specific directory (`~/.achat/agents/<name>`).
pub fn agent_dir(name: &str) -> PathBuf {
    base_dir().join("agents").join(name)
}

/// Return the messages directory for an agent.
pub fn messages_dir(name: &str) -> PathBuf {
    agent_dir(name).join("messages")
}

/// Ensure all directories exist for an agent.
pub fn init_storage(name: &str) -> io::Result<()> {
    fs::create_dir_all(messages_dir(name))
}

/// Determine the JSONL filename for a message target.
fn target_file(agent_name: &str, target: &crate::protocol::Target, from: &str) -> PathBuf {
    let dir = messages_dir(agent_name);
    match target {
        crate::protocol::Target::Direct(peer) => {
            // Normalize: always use the OTHER agent's name
            let peer_name = if peer == agent_name { from } else { peer };
            dir.join(format!("@{peer_name}.jsonl"))
        }
        crate::protocol::Target::Group(group) => dir.join(format!("{group}.jsonl")),
        crate::protocol::Target::Broadcast => dir.join("_broadcast.jsonl"),
    }
}

/// Append a message to the appropriate JSONL file.
pub fn append_message(agent_name: &str, msg: &Message) -> io::Result<()> {
    let path = target_file(agent_name, &msg.to, &msg.from);
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line =
        serde_json::to_string(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    writeln!(file, "{line}")
}

/// Sort messages by timestamp and keep only the last `limit` entries.
/// Returns `(truncated_msgs, total_before_truncation)`.
fn truncate_recent(mut msgs: Vec<Message>, limit: usize) -> (Vec<Message>, usize) {
    msgs.sort_by(|a, b| a.ts.cmp(&b.ts));
    let total = msgs.len();
    if msgs.len() > limit {
        msgs = msgs.split_off(msgs.len() - limit);
    }
    (msgs, total)
}

/// Read messages from a specific JSONL file, with limit.
fn read_file(path: &Path, limit: usize) -> io::Result<(Vec<Message>, usize)> {
    if !path.exists() {
        return Ok((vec![], 0));
    }
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut all: Vec<Message> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(msg) = serde_json::from_str::<Message>(&line) {
            all.push(msg);
        }
    }
    Ok(truncate_recent(all, limit))
}

/// Read direct messages (inbox).
pub fn read_inbox(agent_name: &str, limit: usize) -> io::Result<(Vec<Message>, usize)> {
    let dir = messages_dir(agent_name);
    if !dir.exists() {
        return Ok((vec![], 0));
    }
    let mut all: Vec<Message> = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let fname = entry.file_name();
        let fname = fname.to_string_lossy();
        // Direct messages start with @
        if fname.starts_with('@') && fname.ends_with(".jsonl") {
            let (msgs, _) = read_file(&entry.path(), usize::MAX)?;
            // Only include messages sent TO us (not ones we sent)
            for m in msgs {
                if matches!(&m.to, crate::protocol::Target::Direct(_)) && m.from != agent_name {
                    all.push(m);
                }
            }
        }
    }
    Ok(truncate_recent(all, limit))
}

/// Read broadcast messages (feed).
pub fn read_feed(agent_name: &str, limit: usize) -> io::Result<(Vec<Message>, usize)> {
    let path = messages_dir(agent_name).join("_broadcast.jsonl");
    read_file(&path, limit)
}

/// Read message history for a specific target or all.
pub fn read_log(
    agent_name: &str,
    target: Option<&str>,
    limit: usize,
) -> io::Result<(Vec<Message>, usize)> {
    if let Some(t) = target {
        let path = messages_dir(agent_name).join(format!("{t}.jsonl"));
        read_file(&path, limit)
    } else {
        // All messages from all files
        let dir = messages_dir(agent_name);
        if !dir.exists() {
            return Ok((vec![], 0));
        }
        let mut all: Vec<Message> = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) == Some("jsonl") {
                let (msgs, _) = read_file(&entry.path(), usize::MAX)?;
                all.extend(msgs);
            }
        }
        all.sort_by(|a, b| a.ts.cmp(&b.ts));
        all.dedup_by(|a, b| a.id == b.id);
        Ok(truncate_recent(all, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Message, Target};

    fn make_msg(id: &str, from: &str, to: Target, content: &str, ts: &str) -> Message {
        Message {
            id: id.into(),
            from: from.into(),
            to,
            content: content.into(),
            ts: ts.into(),
        }
    }

    /// Write messages to a temp JSONL file and read them back using `read_file`.
    #[test]
    fn read_file_roundtrip() {
        let dir = std::env::temp_dir().join(format!("achat-rf-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.jsonl");

        for i in 0..5 {
            let msg = make_msg(
                &format!("m{i}"),
                "bob",
                Target::Broadcast,
                &format!("msg {i}"),
                &format!("2026-01-01T10:0{i}:00Z"),
            );
            let mut f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .unwrap();
            writeln!(f, "{}", serde_json::to_string(&msg).unwrap()).unwrap();
        }

        let (msgs, total) = read_file(&path, 3).unwrap();
        assert_eq!(total, 5);
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].content, "msg 2"); // last 3
        assert_eq!(msgs[2].content, "msg 4");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_file_nonexistent_returns_empty() {
        let (msgs, total) = read_file(Path::new("/tmp/nonexistent-achat.jsonl"), 10).unwrap();
        assert_eq!(total, 0);
        assert!(msgs.is_empty());
    }

    /// Test `target_file` routing logic.
    #[test]
    fn target_file_routing() {
        let dir = messages_dir("test-agent");

        let dm = target_file("test-agent", &Target::Direct("bob".into()), "alice");
        assert_eq!(dm, dir.join("@bob.jsonl"));

        // When we are the target, use sender's name
        let dm_to_me = target_file("test-agent", &Target::Direct("test-agent".into()), "alice");
        assert_eq!(dm_to_me, dir.join("@alice.jsonl"));

        let group = target_file("test-agent", &Target::Group("backend".into()), "alice");
        assert_eq!(group, dir.join("backend.jsonl"));

        let bc = target_file("test-agent", &Target::Broadcast, "alice");
        assert_eq!(bc, dir.join("_broadcast.jsonl"));
    }

    #[test]
    fn truncate_recent_empty() {
        let (msgs, total) = truncate_recent(vec![], 10);
        assert!(msgs.is_empty());
        assert_eq!(total, 0);
    }

    #[test]
    fn truncate_recent_under_limit() {
        let items: Vec<Message> = (0..3)
            .map(|i| {
                make_msg(
                    &format!("m{i}"),
                    "a",
                    Target::Broadcast,
                    "x",
                    &format!("2026-01-01T00:0{i}:00Z"),
                )
            })
            .collect();
        let (msgs, total) = truncate_recent(items, 10);
        assert_eq!(msgs.len(), 3);
        assert_eq!(total, 3);
    }

    #[test]
    fn truncate_recent_exact_limit() {
        let items: Vec<Message> = (0..5)
            .map(|i| {
                make_msg(
                    &format!("m{i}"),
                    "a",
                    Target::Broadcast,
                    "x",
                    &format!("2026-01-01T00:0{i}:00Z"),
                )
            })
            .collect();
        let (msgs, total) = truncate_recent(items, 5);
        assert_eq!(msgs.len(), 5);
        assert_eq!(total, 5);
    }
}
