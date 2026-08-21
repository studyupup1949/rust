//! # AATP — AI Agent Transport Protocol
//!
//! A tiny, dependency-free binary framing protocol for LLM agents — a compact binary
//! alternative to text-based RPC that is structurally cheaper to parse (no tokenizing, no
//! per-field allocation) and memory-safe by construction (`#![forbid(unsafe_code)]`,
//! `no_std`-compatible). A frame is a fixed shape:
//!
//! ```text
//! [ Magic "AATP" (4) ][ Version (1) ][ Type (1) ][ Length (4, LE) ][ Payload (N) ][ CRC32C (4, LE) ]
//! ```
//!
//! **Zero-copy / zero-allocation.** [`decode`] borrows the payload straight out of
//! the input buffer (`&[u8]`, no heap copy); [`encode`] writes into a caller-owned
//! `&mut [u8]` (no allocation). Transmute-casting a byte slice to a `repr(C)` struct
//! would need `unsafe`; this crate forbids it and reads each field in place with
//! `u32::from_le_bytes` behind explicit bounds guards instead — the safe form of
//! zero-copy.
//!
//! **Hardened.** Every field is bounds-checked with `slice::get` (a range read that
//! fails closed rather than panics); a truncated frame, a bad magic, an unsupported
//! version, an over-large declared length, or a corrupt payload each return a
//! distinct [`Error`] — never a panic, never an out-of-bounds read.
//!
//! The CRC32C detects **accidental corruption**, not a malicious sender: an attacker can
//! recompute a valid checksum for any payload, so it is integrity-against-noise, **not
//! authentication**. Pair AATP with a MAC/AEAD if you need to trust the peer.
#![no_std]
#![forbid(unsafe_code)]

#[cfg(test)]
extern crate std;

pub mod codec;
pub mod dict;
pub mod message;
pub mod shadow;
pub mod state;

/// Define an agent-message type and generate its zero-copy binary codec — the
/// derive-style ergonomics of the field, with zero dependencies. Fields are declared
/// with a small type vocabulary (`u8`/`u16`/`u32`/`u64`/`str`/`bytes`), and the macro
/// emits the struct plus `encode`/`decode` built on [`codec::Writer`]/[`codec::Reader`],
/// so no byte is hand-indexed:
///
/// ```
/// aatp::aatp_message! {
///     pub struct Turn<'a> {
///         id: u64,
///         role: u8,
///         name: str,
///         content: str,
///     }
/// }
/// let src = Turn { id: 7, role: 2, name: "planner", content: "done" };
/// let mut buf = [0u8; 64];
/// let n = src.encode(&mut buf).unwrap();
/// assert_eq!(Turn::decode(&buf[..n]).unwrap(), src);
/// ```
#[macro_export]
macro_rules! aatp_message {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident<$lt:lifetime> {
            $($field:ident : $kind:ident),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis struct $name<$lt> {
            $(pub $field: $crate::__aatp_field_ty!($lt, $kind)),*
        }

        impl<$lt> $name<$lt> {
            /// Encode into `out`, returning the number of bytes written.
            $vis fn encode(&self, out: &mut [u8]) -> ::core::result::Result<usize, $crate::codec::CodecError> {
                let mut w = $crate::codec::Writer::new(out);
                $( $crate::__aatp_write!(w, self.$field, $kind); )*
                ::core::result::Result::Ok(w.position())
            }
            /// Decode from `buf`, borrowing any `str`/`bytes` fields zero-copy.
            $vis fn decode(buf: &$lt [u8]) -> ::core::result::Result<Self, $crate::codec::CodecError> {
                let mut r = $crate::codec::Reader::new(buf);
                ::core::result::Result::Ok(Self {
                    $( $field: $crate::__aatp_read!(r, $kind) ),*
                })
            }
        }
    };

    // All-scalar variant (no borrowed fields, so no lifetime — avoids an unused-lifetime
    // error for `Ping`/`Ack`/`Heartbeat`-style messages). `str`/`bytes` need the lifetime
    // form above; using one here is a macro error by construction.
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:ident : $kind:ident),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis struct $name {
            $(pub $field: $crate::__aatp_scalar_ty!($kind)),*
        }

        impl $name {
            /// Encode into `out`, returning the number of bytes written.
            $vis fn encode(&self, out: &mut [u8]) -> ::core::result::Result<usize, $crate::codec::CodecError> {
                let mut w = $crate::codec::Writer::new(out);
                $( $crate::__aatp_write!(w, self.$field, $kind); )*
                ::core::result::Result::Ok(w.position())
            }
            /// Decode from `buf`.
            $vis fn decode(buf: &[u8]) -> ::core::result::Result<Self, $crate::codec::CodecError> {
                let mut r = $crate::codec::Reader::new(buf);
                ::core::result::Result::Ok(Self {
                    $( $field: $crate::__aatp_read!(r, $kind) ),*
                })
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __aatp_scalar_ty {
    (u8) => {
        u8
    };
    (u16) => {
        u16
    };
    (u32) => {
        u32
    };
    (u64) => {
        u64
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __aatp_field_ty {
    ($lt:lifetime, u8) => { u8 };
    ($lt:lifetime, u16) => { u16 };
    ($lt:lifetime, u32) => { u32 };
    ($lt:lifetime, u64) => { u64 };
    ($lt:lifetime, str) => { &$lt str };
    ($lt:lifetime, bytes) => { &$lt [u8] };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __aatp_write {
    ($w:ident, $v:expr, u8) => {
        $w.u8($v)?
    };
    ($w:ident, $v:expr, u16) => {
        $w.u16($v)?
    };
    ($w:ident, $v:expr, u32) => {
        $w.u32($v)?
    };
    ($w:ident, $v:expr, u64) => {
        $w.u64($v)?
    };
    ($w:ident, $v:expr, str) => {
        $w.str($v)?
    };
    ($w:ident, $v:expr, bytes) => {
        $w.bytes($v)?
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __aatp_read {
    ($r:ident, u8) => {
        $r.u8()?
    };
    ($r:ident, u16) => {
        $r.u16()?
    };
    ($r:ident, u32) => {
        $r.u32()?
    };
    ($r:ident, u64) => {
        $r.u64()?
    };
    ($r:ident, str) => {
        $r.str()?
    };
    ($r:ident, bytes) => {
        $r.bytes()?
    };
}

/// The 4-byte frame magic: ASCII `AATP`.
pub const MAGIC: [u8; 4] = *b"AATP";

/// The only wire version this build speaks.
pub const VERSION: u8 = 1;

/// Bytes before the payload: magic (4) + version (1) + type (1) + length (4).
pub const HEADER_LEN: usize = 10;

/// The trailing CRC32C, in bytes.
pub const CHECKSUM_LEN: usize = 4;

/// Non-payload bytes in a frame: [`HEADER_LEN`] + [`CHECKSUM_LEN`].
pub const OVERHEAD: usize = HEADER_LEN + CHECKSUM_LEN;

/// The largest payload a frame may declare or carry — the guard against a hostile
/// length field asking us to address gigabytes. 16 MiB is far above any real agent
/// message and keeps `HEADER_LEN + length + CHECKSUM_LEN` well inside a 32-bit
/// `usize`, so the frame arithmetic can never overflow.
pub const MAX_PAYLOAD: usize = 16 * 1024 * 1024;

/// Everything that can go wrong reading or writing a frame. Each variant is a
/// distinct fail-closed outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The buffer is shorter than the header, or shorter than the frame its own
    /// length field declares.
    TooShort,
    /// The first four bytes are not [`MAGIC`].
    BadMagic,
    /// The version byte is not one this build understands.
    UnsupportedVersion(u8),
    /// The declared (or supplied) payload length exceeds [`MAX_PAYLOAD`] — an
    /// invalid-length attack, refused before any addressing.
    LengthTooLarge(usize),
    /// The trailing CRC32C does not match the CRC32C of the header+payload.
    ChecksumMismatch { expected: u32, found: u32 },
    /// [`encode`]'s output buffer is smaller than the frame it must hold.
    BufferTooSmall { need: usize, have: usize },
}

/// A decoded frame. Borrows its payload from the input buffer — decoding allocates
/// nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packet<'a> {
    version: u8,
    msg_type: u8,
    payload: &'a [u8],
}

impl<'a> Packet<'a> {
    /// The frame's protocol version (always [`VERSION`] for a decoded frame).
    pub fn version(&self) -> u8 {
        self.version
    }
    /// The application-defined message-type byte.
    pub fn msg_type(&self) -> u8 {
        self.msg_type
    }
    /// The payload bytes, borrowed from the buffer [`decode`] was given.
    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

/// The single length guard, shared by [`encode`] and [`decode`] so the boundary is
/// pinned once, in a pure function a test can hit without allocating [`MAX_PAYLOAD`]
/// bytes. `len == MAX_PAYLOAD` is allowed; one more is refused.
fn check_payload_len(len: usize) -> Result<(), Error> {
    if len > MAX_PAYLOAD {
        Err(Error::LengthTooLarge(len))
    } else {
        Ok(())
    }
}

/// Encode a `(msg_type, payload)` into `out`, returning the number of bytes written.
/// Allocation-free: the frame is laid down directly in the caller's buffer.
pub fn encode(msg_type: u8, payload: &[u8], out: &mut [u8]) -> Result<usize, Error> {
    check_payload_len(payload.len())?;
    let frame_len = OVERHEAD + payload.len();
    if out.len() < frame_len {
        return Err(Error::BufferTooSmall {
            need: frame_len,
            have: out.len(),
        });
    }
    out[0..4].copy_from_slice(&MAGIC);
    out[4] = VERSION;
    out[5] = msg_type;
    // `payload.len() <= MAX_PAYLOAD < u32::MAX`, so this cast cannot truncate.
    out[6..10].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    let payload_end = HEADER_LEN + payload.len();
    out[HEADER_LEN..payload_end].copy_from_slice(payload);
    let checksum = crc32c(&out[..payload_end]);
    out[payload_end..payload_end + CHECKSUM_LEN].copy_from_slice(&checksum.to_le_bytes());
    Ok(frame_len)
}

/// Decode one frame from the front of `buf`, borrowing its payload. `buf` may be
/// longer than the frame (a stream): only `HEADER_LEN + length + CHECKSUM_LEN` bytes
/// are consumed and validated. Every access is a fail-closed `slice::get`; the
/// CRC32C is verified last.
pub fn decode(buf: &[u8]) -> Result<Packet<'_>, Error> {
    let header = buf.get(0..HEADER_LEN).ok_or(Error::TooShort)?;
    if header[0..4] != MAGIC {
        return Err(Error::BadMagic);
    }
    let version = header[4];
    if version != VERSION {
        return Err(Error::UnsupportedVersion(version));
    }
    let msg_type = header[5];
    let length = u32::from_le_bytes([header[6], header[7], header[8], header[9]]) as usize;
    check_payload_len(length)?;
    let payload_end = HEADER_LEN + length;
    let payload = buf.get(HEADER_LEN..payload_end).ok_or(Error::TooShort)?;
    let checksum_bytes = buf
        .get(payload_end..payload_end + CHECKSUM_LEN)
        .ok_or(Error::TooShort)?;
    let found = u32::from_le_bytes([
        checksum_bytes[0],
        checksum_bytes[1],
        checksum_bytes[2],
        checksum_bytes[3],
    ]);
    let expected = crc32c(&buf[..payload_end]);
    if found != expected {
        return Err(Error::ChecksumMismatch { expected, found });
    }
    Ok(Packet {
        version,
        msg_type,
        payload,
    })
}

/// The 256-entry CRC32C lookup table, built once at COMPILE TIME by [`crc32c_table`]
/// — so the checksum is table-fast yet the crate ships no data blob and stays
/// `no_std`/zero-dependency.
static CRC32C_TABLE: [u32; 256] = crc32c_table();

/// Compute the CRC32C table in a `const fn` (reflected poly `0x82F6_3B78`). Every entry
/// is derived from the same shift/mask logic, so any mutation of *that logic* corrupts
/// every entry at once and is caught by the [`crc32c`] known-answer test. A single WRONG
/// entry (a typo the operator-level prober cannot express) is a separate risk, closed by
/// `crc32c_table_is_pinned_entry_for_entry`, which folds all 256 entries into one check.
const fn crc32c_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut crc = n as u32;
        let mut bit = 0;
        while bit < 8 {
            // `-(crc & 1)` is `0xFFFF_FFFF` when the low bit is set, else `0`.
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0x82F6_3B78 & mask);
            bit += 1;
        }
        table[n] = crc;
        n += 1;
    }
    table
}

/// CRC32C (Castagnoli, reflected poly `0x82F6_3B78`, init/xorout `0xFFFF_FFFF`) — the
/// checksum of iSCSI and SSE4.2's `crc32` instruction, here table-driven (one byte
/// per step). Pinned by the standard `"123456789" -> 0xE306_9283` known-answer vector.
pub fn crc32c(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    let mut i = 0;
    while i < data.len() {
        let index = ((crc ^ data[i] as u32) & 0xFF) as usize;
        crc = CRC32C_TABLE[index] ^ (crc >> 8);
        i += 1;
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_constants_have_their_specified_values() {
        // Pin the frame geometry to its literal values, so a mutation of any constant
        // expression (e.g. `16 * 1024 * 1024` → `16 / 1024 * 1024`, or `HEADER_LEN +
        // CHECKSUM_LEN` → `-`) is caught here rather than as a downstream surprise.
        assert_eq!(MAGIC, *b"AATP");
        assert_eq!(VERSION, 1);
        assert_eq!(HEADER_LEN, 10);
        assert_eq!(CHECKSUM_LEN, 4);
        assert_eq!(OVERHEAD, 14);
        assert_eq!(MAX_PAYLOAD, 16_777_216); // 16 MiB
    }

    #[test]
    fn crc32c_matches_the_canonical_vectors() {
        assert_eq!(crc32c(b""), 0x0000_0000);
        assert_eq!(crc32c(b"123456789"), 0xE306_9283); // the canonical CRC32C check value
    }

    #[test]
    fn crc32c_table_is_pinned_entry_for_entry() {
        // The known-answer vectors above only touch a handful of the 256 entries, so a
        // single WRONG entry (e.g. a typo in the const generator that the operator-level
        // prober cannot express) could slip past them. Fold every entry into one order-
        // and value-sensitive digest, computed WITHOUT the CRC logic itself (no
        // circularity), so any single corrupted entry changes the digest.
        let digest = CRC32C_TABLE
            .iter()
            .fold(0x811C_9DC5u32, |acc, &e| acc.wrapping_mul(0x0100_0193) ^ e);
        assert_eq!(digest, 0x4B64_0B45);
    }

    #[test]
    fn crc32c_is_order_and_length_sensitive() {
        assert_ne!(crc32c(b"ab"), crc32c(b"ba")); // order
        assert_ne!(crc32c(b"a"), crc32c(b"aa")); // length
        assert_ne!(crc32c(&[0x00]), crc32c(b"")); // a single zero byte still changes it
        assert_ne!(crc32c(&[0xFF]), crc32c(&[0x00])); // high bits reach different states
    }

    #[test]
    fn check_payload_len_allows_up_to_the_cap_and_refuses_one_more() {
        assert_eq!(check_payload_len(0), Ok(()));
        assert_eq!(check_payload_len(MAX_PAYLOAD), Ok(())); // the cap itself is allowed
        assert_eq!(
            check_payload_len(MAX_PAYLOAD + 1),
            Err(Error::LengthTooLarge(MAX_PAYLOAD + 1))
        );
    }

    /// The behavioural spine: encode then decode returns the input, for payloads
    /// that exercise the empty, one-chunk, and multi-byte cases.
    #[test]
    fn round_trip_preserves_type_and_payload() {
        for payload in [
            &b""[..],
            &b"x"[..],
            &b"hello agent"[..],
            &[0u8, 1, 2, 3, 255, 254][..],
        ] {
            let mut buf = std::vec![0u8; OVERHEAD + payload.len()];
            let n = encode(0xA7, payload, &mut buf).unwrap();
            assert_eq!(n, OVERHEAD + payload.len());
            let pkt = decode(&buf).unwrap();
            assert_eq!(pkt.version(), VERSION);
            assert_eq!(pkt.msg_type(), 0xA7);
            assert_eq!(pkt.payload(), payload);
        }
    }

    /// The exact on-wire byte layout — pins every offset and the magic/version bytes.
    #[test]
    fn encodes_the_exact_wire_layout() {
        let mut buf = [0u8; OVERHEAD + 2];
        let n = encode(0x09, &[0xAA, 0xBB], &mut buf).unwrap();
        assert_eq!(n, 16);
        assert_eq!(&buf[0..4], b"AATP"); // magic
        assert_eq!(buf[4], 1); // version
        assert_eq!(buf[5], 0x09); // type
        assert_eq!(&buf[6..10], &[2, 0, 0, 0]); // length = 2, little-endian
        assert_eq!(&buf[10..12], &[0xAA, 0xBB]); // payload
        let want_crc = crc32c(&buf[0..12]);
        assert_eq!(&buf[12..16], &want_crc.to_le_bytes()); // checksum over header+payload
    }

    #[test]
    fn encode_rejects_a_too_small_buffer_at_the_exact_boundary() {
        let payload = [1u8, 2, 3];
        let need = OVERHEAD + payload.len();
        let mut short = std::vec![0u8; need - 1];
        assert_eq!(
            encode(0, &payload, &mut short),
            Err(Error::BufferTooSmall {
                need,
                have: need - 1
            })
        );
        let mut exact = std::vec![0u8; need]; // exactly enough must succeed
        assert_eq!(encode(0, &payload, &mut exact), Ok(need));
    }

    #[test]
    fn decode_rejects_a_header_shorter_than_ten_bytes() {
        for len in 0..HEADER_LEN {
            assert_eq!(decode(&std::vec![0u8; len]), Err(Error::TooShort));
        }
        let mut smallest = [0u8; OVERHEAD]; // header + zero payload + checksum is the minimum valid frame
        encode(0, &[], &mut smallest).unwrap();
        assert!(decode(&smallest).is_ok());
    }

    #[test]
    fn decode_rejects_bad_magic() {
        let mut buf = [0u8; OVERHEAD];
        encode(0, &[], &mut buf).unwrap();
        buf[0] = b'X';
        assert_eq!(decode(&buf), Err(Error::BadMagic));
    }

    #[test]
    fn decode_rejects_an_unsupported_version_before_checking_the_checksum() {
        let mut buf = [0u8; OVERHEAD];
        encode(0, &[], &mut buf).unwrap();
        buf[4] = 2; // also invalidates the checksum, but the version guard runs first
        assert_eq!(decode(&buf), Err(Error::UnsupportedVersion(2)));
    }

    #[test]
    fn decode_rejects_an_over_large_declared_length() {
        let mut buf = [0u8; OVERHEAD];
        encode(0, &[], &mut buf).unwrap();
        buf[6..10].copy_from_slice(&((MAX_PAYLOAD as u32) + 1).to_le_bytes());
        assert_eq!(decode(&buf), Err(Error::LengthTooLarge(MAX_PAYLOAD + 1)));
    }

    #[test]
    fn decode_rejects_a_frame_truncated_below_its_declared_length() {
        let mut buf = std::vec![0u8; HEADER_LEN + 3]; // declares 5 bytes, supplies 3
        buf[0..4].copy_from_slice(&MAGIC);
        buf[4] = VERSION;
        buf[6..10].copy_from_slice(&5u32.to_le_bytes());
        assert_eq!(decode(&buf), Err(Error::TooShort));
    }

    #[test]
    fn decode_rejects_a_frame_missing_its_checksum_bytes() {
        // header + 3-byte payload but NO room for the 4-byte checksum
        let mut buf = std::vec![0u8; HEADER_LEN + 3];
        buf[0..4].copy_from_slice(&MAGIC);
        buf[4] = VERSION;
        buf[6..10].copy_from_slice(&3u32.to_le_bytes());
        assert_eq!(decode(&buf), Err(Error::TooShort));
    }

    #[test]
    fn decode_rejects_a_corrupt_payload_via_the_checksum() {
        let mut buf = std::vec![0u8; OVERHEAD + 3];
        encode(0x11, &[7, 8, 9], &mut buf).unwrap();
        assert!(decode(&buf).is_ok());
        // `expected` is the CRC of the (now corrupt) header+payload; `found` is the
        // stored trailer, unchanged. Assert BOTH exact values so a transposition of the
        // two fields (which no operator mutation expresses, but a refactor could) is caught.
        let found = u32::from_le_bytes([buf[13], buf[14], buf[15], buf[16]]);
        buf[HEADER_LEN] ^= 0xFF; // flip a payload byte
        let expected = crc32c(&buf[..13]);
        assert_eq!(
            decode(&buf),
            Err(Error::ChecksumMismatch { expected, found })
        );
    }

    #[test]
    fn decode_reads_only_the_frame_and_ignores_trailing_stream_bytes() {
        let mut buf = std::vec![0u8; OVERHEAD + 3 + 4]; // 4 extra trailing bytes
        encode(0x22, &[1, 2, 3], &mut buf[..OVERHEAD + 3]).unwrap();
        buf[OVERHEAD + 3..].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        let pkt = decode(&buf).unwrap();
        assert_eq!(pkt.payload(), &[1, 2, 3]);
    }
}

#[cfg(test)]
mod macro_tests {
    // A message type defined entirely by the macro — no hand-written codec. Exercises
    // EVERY field kind, `u16` included, so the macro's `u16` arm is actually covered.
    crate::aatp_message! {
        pub struct GenMsg<'a> {
            id: u64,
            role: u8,
            flags: u16,
            tokens: u32,
            name: str,
            blob: bytes,
        }
    }

    // The all-scalar (no-lifetime) arm — a message with no borrowed fields.
    crate::aatp_message! {
        pub struct Ping {
            seq: u64,
            code: u16,
        }
    }

    #[test]
    fn macro_generated_codec_round_trips() {
        let src = GenMsg {
            id: 0x0102_0304_0506_0708,
            role: 9,
            flags: 0xBEEF,
            tokens: 4242,
            name: "web_search",
            blob: &[0xDE, 0xAD, 0xBE, 0xEF],
        };
        let mut buf = [0u8; 128];
        let n = src.encode(&mut buf).unwrap();
        let got = GenMsg::decode(&buf[..n]).unwrap();
        assert_eq!(got, src);
        assert_eq!(got.flags, 0xBEEF); // pins the u16 field maps to the u16 codec, not u32
    }

    #[test]
    fn macro_all_scalar_struct_round_trips() {
        let p = Ping {
            seq: 0x1122_3344_5566_7788,
            code: 0x0102,
        };
        let mut buf = [0u8; 16];
        let n = p.encode(&mut buf).unwrap();
        assert_eq!(n, 10); // u64 + u16
        assert_eq!(Ping::decode(&buf[..n]).unwrap(), p);
    }

    #[test]
    fn macro_generated_decode_fails_closed_on_truncation() {
        assert!(GenMsg::decode(&[0u8; 3]).is_err()); // not even the u64 id fits
        assert!(Ping::decode(&[0u8; 3]).is_err());
    }
}
