//! A complex agent-message payload for AATP — the shape an LLM agent actually
//! ships: a message id, a role byte, a tool/agent name, free-text content, and a
//! token count. Encoded as a compact little-endian binary record with
//! length-prefixed strings, and **decoded zero-copy**: `name` and `content` are
//! borrowed `&str` slices validated in place, no allocation.
//!
//! Wire layout:
//! ```text
//! [ id (8, LE) ][ role (1) ][ token_count (4, LE) ][ name_len (2, LE) ][ name (…) ]
//! [ content_len (4, LE) ][ content (…) ]
//! ```

use core::str;

/// Longest permitted `name`, in bytes — a bounded, testable cap (also keeps the
/// `u16` length prefix from ever truncating).
pub const MAX_NAME: usize = 1024;

/// Longest permitted `content`, in bytes — 1 MiB, well under `u32::MAX` so the
/// length prefix cannot truncate and the offsets cannot overflow `usize`.
pub const MAX_CONTENT: usize = 1024 * 1024;

const ID_LEN: usize = 8;
const ROLE_LEN: usize = 1;
const TOKENS_LEN: usize = 4;
const NAME_LEN_LEN: usize = 2;
const CONTENT_LEN_LEN: usize = 4;
/// Fixed bytes before the name: id + role + token_count + name_len.
const FIXED_PREFIX: usize = ID_LEN + ROLE_LEN + TOKENS_LEN + NAME_LEN_LEN;

/// Everything that can go wrong reading or writing an [`AgentMessage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgError {
    /// The buffer ends before a field the record's own lengths require.
    TooShort,
    /// A string field is not valid UTF-8.
    BadUtf8,
    /// `name` is longer than [`MAX_NAME`].
    NameTooLong(usize),
    /// `content` is longer than [`MAX_CONTENT`].
    ContentTooLong(usize),
    /// [`encode`]'s output buffer is smaller than the record needs.
    BufferTooSmall { need: usize, have: usize },
}

/// One agent message. `name`/`content` borrow from the decode buffer — zero-copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentMessage<'a> {
    /// Monotonic message id.
    pub id: u64,
    /// Role/type byte (e.g. system/user/assistant/tool — the application's own enum).
    pub role: u8,
    /// Tool or agent name.
    pub name: &'a str,
    /// The message text.
    pub content: &'a str,
    /// Token usage for this message.
    pub token_count: u32,
}

fn check_name_len(len: usize) -> Result<(), MsgError> {
    if len > MAX_NAME {
        Err(MsgError::NameTooLong(len))
    } else {
        Ok(())
    }
}

fn check_content_len(len: usize) -> Result<(), MsgError> {
    if len > MAX_CONTENT {
        Err(MsgError::ContentTooLong(len))
    } else {
        Ok(())
    }
}

/// The exact number of bytes [`encode`] will write for `msg`.
pub fn encoded_len(msg: &AgentMessage) -> usize {
    FIXED_PREFIX + msg.name.len() + CONTENT_LEN_LEN + msg.content.len()
}

/// Encode `msg` into `out`, returning the byte count written. Allocation-free.
pub fn encode(msg: &AgentMessage, out: &mut [u8]) -> Result<usize, MsgError> {
    check_name_len(msg.name.len())?;
    check_content_len(msg.content.len())?;
    let need = encoded_len(msg);
    if out.len() < need {
        return Err(MsgError::BufferTooSmall {
            need,
            have: out.len(),
        });
    }
    out[0..8].copy_from_slice(&msg.id.to_le_bytes());
    out[8] = msg.role;
    out[9..13].copy_from_slice(&msg.token_count.to_le_bytes());
    // `name.len() <= MAX_NAME < u16::MAX`, so this cast cannot truncate.
    out[13..15].copy_from_slice(&(msg.name.len() as u16).to_le_bytes());
    let name_end = FIXED_PREFIX + msg.name.len();
    out[FIXED_PREFIX..name_end].copy_from_slice(msg.name.as_bytes());
    let clen_end = name_end + CONTENT_LEN_LEN;
    // `content.len() <= MAX_CONTENT < u32::MAX`, so this cast cannot truncate.
    out[name_end..clen_end].copy_from_slice(&(msg.content.len() as u32).to_le_bytes());
    let content_end = clen_end + msg.content.len();
    out[clen_end..content_end].copy_from_slice(msg.content.as_bytes());
    Ok(need)
}

/// Decode one [`AgentMessage`] from the front of `buf`, borrowing its strings.
/// Every access is a fail-closed `slice::get`; both strings are UTF-8-validated.
pub fn decode(buf: &[u8]) -> Result<AgentMessage<'_>, MsgError> {
    let fixed = buf.get(0..FIXED_PREFIX).ok_or(MsgError::TooShort)?;
    let id = u64::from_le_bytes([
        fixed[0], fixed[1], fixed[2], fixed[3], fixed[4], fixed[5], fixed[6], fixed[7],
    ]);
    let role = fixed[8];
    let token_count = u32::from_le_bytes([fixed[9], fixed[10], fixed[11], fixed[12]]);
    let name_len = u16::from_le_bytes([fixed[13], fixed[14]]) as usize;
    check_name_len(name_len)?;
    let name_end = FIXED_PREFIX + name_len;
    let name_bytes = buf.get(FIXED_PREFIX..name_end).ok_or(MsgError::TooShort)?;
    let name = str::from_utf8(name_bytes).map_err(|_| MsgError::BadUtf8)?;
    let clen_end = name_end + CONTENT_LEN_LEN;
    let clen_bytes = buf.get(name_end..clen_end).ok_or(MsgError::TooShort)?;
    let content_len =
        u32::from_le_bytes([clen_bytes[0], clen_bytes[1], clen_bytes[2], clen_bytes[3]]) as usize;
    check_content_len(content_len)?;
    let content_end = clen_end + content_len;
    let content_bytes = buf.get(clen_end..content_end).ok_or(MsgError::TooShort)?;
    let content = str::from_utf8(content_bytes).map_err(|_| MsgError::BadUtf8)?;
    Ok(AgentMessage {
        id,
        role,
        name,
        content,
        token_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AgentMessage<'static> {
        AgentMessage {
            id: 0x0102_0304_0506_0708,
            role: 3,
            name: "web_search",
            content: "find the latest release notes",
            token_count: 4242,
        }
    }

    #[test]
    fn fixed_prefix_has_its_specified_geometry() {
        // pins ID/ROLE/TOKENS/NAME_LEN and the `+` that sums them (mutant `-` ≠ 15)
        assert_eq!(FIXED_PREFIX, 15);
        assert_eq!(MAX_NAME, 1024);
        assert_eq!(MAX_CONTENT, 1024 * 1024);
    }

    #[test]
    fn round_trip_preserves_every_field() {
        let msg = sample();
        let mut buf = std::vec![0u8; encoded_len(&msg)];
        let n = encode(&msg, &mut buf).unwrap();
        assert_eq!(n, buf.len());
        assert_eq!(decode(&buf).unwrap(), msg);
    }

    #[test]
    fn round_trip_handles_empty_strings_and_multibyte_utf8() {
        for (name, content) in [
            ("", ""),
            ("x", ""),
            ("", "y"),
            ("café", "日本語のテキスト — π ≈ 3.14"),
        ] {
            let msg = AgentMessage {
                id: 7,
                role: 1,
                name,
                content,
                token_count: 0,
            };
            let mut buf = std::vec![0u8; encoded_len(&msg)];
            encode(&msg, &mut buf).unwrap();
            let got = decode(&buf).unwrap();
            assert_eq!(got.name, name);
            assert_eq!(got.content, content);
        }
    }

    #[test]
    fn encodes_the_exact_wire_layout() {
        let msg = AgentMessage {
            id: 1,
            role: 2,
            name: "ab",
            content: "cd",
            token_count: 0x11,
        };
        let mut buf = std::vec![0u8; encoded_len(&msg)];
        encode(&msg, &mut buf).unwrap();
        assert_eq!(&buf[0..8], &1u64.to_le_bytes()); // id
        assert_eq!(buf[8], 2); // role
        assert_eq!(&buf[9..13], &0x11u32.to_le_bytes()); // token_count
        assert_eq!(&buf[13..15], &2u16.to_le_bytes()); // name_len = 2
        assert_eq!(&buf[15..17], b"ab"); // name
        assert_eq!(&buf[17..21], &2u32.to_le_bytes()); // content_len = 2
        assert_eq!(&buf[21..23], b"cd"); // content
        assert_eq!(encoded_len(&msg), 23);
    }

    #[test]
    fn check_len_helpers_allow_the_cap_and_refuse_one_more() {
        assert_eq!(check_name_len(MAX_NAME), Ok(()));
        assert_eq!(
            check_name_len(MAX_NAME + 1),
            Err(MsgError::NameTooLong(MAX_NAME + 1))
        );
        assert_eq!(check_content_len(MAX_CONTENT), Ok(()));
        assert_eq!(
            check_content_len(MAX_CONTENT + 1),
            Err(MsgError::ContentTooLong(MAX_CONTENT + 1))
        );
    }

    #[test]
    fn encode_rejects_a_too_small_buffer_at_the_exact_boundary() {
        let msg = sample();
        let need = encoded_len(&msg);
        let mut short = std::vec![0u8; need - 1];
        assert_eq!(
            encode(&msg, &mut short),
            Err(MsgError::BufferTooSmall {
                need,
                have: need - 1
            })
        );
        let mut exact = std::vec![0u8; need];
        assert_eq!(encode(&msg, &mut exact), Ok(need));
    }

    #[test]
    fn decode_rejects_a_truncated_fixed_prefix() {
        for len in 0..FIXED_PREFIX {
            assert_eq!(decode(&std::vec![0u8; len]), Err(MsgError::TooShort));
        }
    }

    #[test]
    fn decode_rejects_a_name_running_past_the_buffer() {
        let mut buf = std::vec![0u8; FIXED_PREFIX];
        buf[13..15].copy_from_slice(&5u16.to_le_bytes()); // declares 5 name bytes, supplies none
        assert_eq!(decode(&buf), Err(MsgError::TooShort));
    }

    #[test]
    fn decode_rejects_content_running_past_the_buffer() {
        let msg = AgentMessage {
            id: 0,
            role: 0,
            name: "n",
            content: "",
            token_count: 0,
        };
        let mut buf = std::vec![0u8; encoded_len(&msg)];
        encode(&msg, &mut buf).unwrap();
        // rewrite content_len to claim 9 bytes that aren't there (content_len sits after the name)
        let clen_at = FIXED_PREFIX + 1;
        buf[clen_at..clen_at + 4].copy_from_slice(&9u32.to_le_bytes());
        assert_eq!(decode(&buf), Err(MsgError::TooShort));
    }

    #[test]
    fn decode_rejects_an_over_large_declared_name_len() {
        let mut buf = std::vec![0u8; FIXED_PREFIX];
        buf[13..15].copy_from_slice(&((MAX_NAME as u16) + 1).to_le_bytes());
        assert_eq!(decode(&buf), Err(MsgError::NameTooLong(MAX_NAME + 1)));
    }

    #[test]
    fn decode_rejects_invalid_utf8_in_a_string_field() {
        let msg = AgentMessage {
            id: 0,
            role: 0,
            name: "n",
            content: "c",
            token_count: 0,
        };
        let mut buf = std::vec![0u8; encoded_len(&msg)];
        encode(&msg, &mut buf).unwrap();
        buf[FIXED_PREFIX] = 0xFF; // corrupt the single name byte into invalid UTF-8
        assert_eq!(decode(&buf), Err(MsgError::BadUtf8));
    }
}
