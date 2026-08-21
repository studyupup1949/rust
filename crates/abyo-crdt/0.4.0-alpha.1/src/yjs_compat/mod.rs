//! Wire-format compatibility with [Yjs](https://docs.yjs.dev) — primitives,
//! state-vector exchange, and a minimal Y.Update v1 snapshot encoder.
//!
//! Yjs documents are serialized with a custom binary encoding (`lib0`)
//! that is **not** an industry-standard format like CBOR or Protobuf.
//! This module ships the primitives most useful to abyo-crdt clients:
//!
//! - [`lib0::write_var_uint`] / [`lib0::read_var_uint`] — Yjs's
//!   little-endian variable-length unsigned integer encoding (the
//!   `lib0/encoding.encodeVarUint` and `decodeVarUint` JS functions).
//! - [`lib0::write_var_int`] / [`lib0::read_var_int`] — signed variant.
//! - [`lib0::write_var_string`] / [`lib0::read_var_string`] — UTF-8
//!   length-prefixed strings.
//! - [`StateVector::encode`] / [`StateVector::decode`] — Y.Doc state
//!   vector format (`Y.encodeStateVector` / `Y.encodeStateVectorFromUpdate`).
//!
//! ## What's NOT here
//!
//! Full Y.Doc update parsing (`Y.encodeStateAsUpdate`,
//! `Y.applyUpdate`) is intentionally out of scope for v0.4 — Yjs's
//! struct-level encoding has many type-specific variants (Text, Map,
//! Array, `XmlElement`…) and replicating it byte-for-byte would be
//! several weeks of careful work. For now, use [`crate::Text::to_delta`]
//! / [`crate::Text::from_delta`] for content-level interop and the
//! state-vector primitives below for handshake-level interop.
//!
//! ## State-vector use case
//!
//! When two replicas — one running Yjs in a browser, one running
//! abyo-crdt on a server — want to negotiate which ops to exchange,
//! they swap their state vectors. The Yjs side encodes its vector
//! using `Y.encodeStateVector(doc)`, sends bytes to the server, and
//! the server decodes them with [`StateVector::decode`]. The server
//! then computes "ops the client doesn't have" and ships them back
//! (as Quill Delta if it's a Y.Text, since full Y.Update is TBD).

use crate::id::ReplicaId;
use crate::version::VersionVector;
use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod update;
pub use update::{snapshot_string_to_yjs_update, snapshot_text_to_yjs_update};

// ---------------------------------------------------------------------------
// lib0 codec
// ---------------------------------------------------------------------------

/// `lib0` low-level primitives: variable-length integers, strings, etc.
///
/// All of these are byte-identical to what the JS `lib0/encoding` library
/// emits and `lib0/decoding` consumes.
pub mod lib0 {
    /// I/O error during lib0 decoding.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Error {
        /// Unexpected end of buffer while decoding.
        Truncated,
        /// Var-int / var-uint exceeded its width.
        Overflow,
        /// String contained invalid UTF-8.
        InvalidUtf8,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::Truncated => f.write_str("lib0: unexpected end of buffer"),
                Error::Overflow => f.write_str("lib0: var-int/var-uint overflow"),
                Error::InvalidUtf8 => f.write_str("lib0: invalid UTF-8 in var-string"),
            }
        }
    }

    impl std::error::Error for Error {}

    /// Append a Yjs-style variable-length unsigned 64-bit integer to `out`.
    ///
    /// Encoding: 7 bits per byte, LSB-first; high bit set on every byte
    /// except the last. `0` encodes as `[0]`; `127` as `[0x7F]`; `128`
    /// as `[0x80, 0x01]`.
    pub fn write_var_uint(out: &mut Vec<u8>, mut n: u64) {
        while n >= 0x80 {
            out.push(((n & 0x7F) | 0x80) as u8);
            n >>= 7;
        }
        out.push(n as u8);
    }

    /// Decode a Yjs-style var-uint from `bytes`, returning `(value,
    /// bytes_consumed)`.
    ///
    /// # Errors
    ///
    /// [`Error::Truncated`] if the buffer ends mid-value.
    /// [`Error::Overflow`] if the encoded value exceeds `u64::MAX`.
    pub fn read_var_uint(bytes: &[u8]) -> Result<(u64, usize), Error> {
        let mut n: u64 = 0;
        let mut shift: u32 = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            if shift >= 64 {
                return Err(Error::Overflow);
            }
            let chunk = u64::from(byte & 0x7F);
            n |= chunk.checked_shl(shift).ok_or(Error::Overflow)?;
            if byte & 0x80 == 0 {
                return Ok((n, i + 1));
            }
            shift += 7;
        }
        Err(Error::Truncated)
    }

    /// Append a Yjs-style signed var-int. Yjs uses a low-bit sign flag:
    /// the LSB of the first byte is the sign bit, the remaining bits are
    /// the magnitude continued by following bytes.
    pub fn write_var_int(out: &mut Vec<u8>, n: i64) {
        let mut value = n.unsigned_abs();
        let sign = if n < 0 { 0x40 } else { 0 };
        // First byte: 6 bits of magnitude + sign bit + continuation bit.
        let cont = if value > 0x3F { 0x80 } else { 0 };
        out.push((value & 0x3F) as u8 | sign | cont);
        value >>= 6;
        while value > 0 {
            let cont = if value > 0x7F { 0x80 } else { 0 };
            out.push((value & 0x7F) as u8 | cont);
            value >>= 7;
        }
    }

    /// Decode a Yjs-style signed var-int from `bytes`.
    pub fn read_var_int(bytes: &[u8]) -> Result<(i64, usize), Error> {
        let first = *bytes.first().ok_or(Error::Truncated)?;
        let sign = first & 0x40 != 0;
        let mut value = u64::from(first & 0x3F);
        let mut shift = 6u32;
        let mut consumed = 1usize;
        if first & 0x80 != 0 {
            for &byte in &bytes[1..] {
                consumed += 1;
                if shift >= 64 {
                    return Err(Error::Overflow);
                }
                let chunk = u64::from(byte & 0x7F);
                value |= chunk.checked_shl(shift).ok_or(Error::Overflow)?;
                shift += 7;
                if byte & 0x80 == 0 {
                    break;
                }
                if consumed >= bytes.len() {
                    return Err(Error::Truncated);
                }
            }
        }
        if value > i64::MAX as u64 {
            return Err(Error::Overflow);
        }
        // Already bounds-checked above, so the cast can't wrap.
        #[allow(clippy::cast_possible_wrap)]
        let unsigned_signed = value as i64;
        let signed = if sign {
            -unsigned_signed
        } else {
            unsigned_signed
        };
        Ok((signed, consumed))
    }

    /// Append a length-prefixed UTF-8 string.
    pub fn write_var_string(out: &mut Vec<u8>, s: &str) {
        let bytes = s.as_bytes();
        write_var_uint(out, bytes.len() as u64);
        out.extend_from_slice(bytes);
    }

    /// Decode a length-prefixed UTF-8 string.
    pub fn read_var_string(bytes: &[u8]) -> Result<(String, usize), Error> {
        let (len, hdr) = read_var_uint(bytes)?;
        let len = len as usize;
        let payload = bytes.get(hdr..hdr + len).ok_or(Error::Truncated)?;
        let s = std::str::from_utf8(payload)
            .map_err(|_| Error::InvalidUtf8)?
            .to_string();
        Ok((s, hdr + len))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn var_uint_round_trip() {
            for v in [0u64, 1, 127, 128, 255, 16384, u64::MAX] {
                let mut buf = Vec::new();
                write_var_uint(&mut buf, v);
                let (decoded, n) = read_var_uint(&buf).unwrap();
                assert_eq!(decoded, v);
                assert_eq!(n, buf.len());
            }
        }

        #[test]
        fn var_int_round_trip() {
            for v in [0i64, 1, -1, 63, -63, 64, -64, 1_000_000, -1_000_000] {
                let mut buf = Vec::new();
                write_var_int(&mut buf, v);
                let (decoded, n) = read_var_int(&buf).unwrap();
                assert_eq!(decoded, v, "round-trip failed for {v}");
                assert_eq!(n, buf.len());
            }
        }

        #[test]
        fn var_string_round_trip() {
            for s in ["", "hello", "👨‍👩‍👧 emoji"] {
                let mut buf = Vec::new();
                write_var_string(&mut buf, s);
                let (decoded, n) = read_var_string(&buf).unwrap();
                assert_eq!(decoded, s);
                assert_eq!(n, buf.len());
            }
        }

        #[test]
        fn var_uint_truncated() {
            let buf = [0x80u8, 0x80]; // continuation bytes with no terminator
            assert_eq!(read_var_uint(&buf).err(), Some(Error::Truncated));
        }

        #[test]
        fn var_uint_known_encoding() {
            // 127 is 7 bits → 1 byte 0x7F.
            let mut buf = Vec::new();
            write_var_uint(&mut buf, 127);
            assert_eq!(buf, vec![0x7F]);
            // 128 is 0b1000_0000 → encoded as [0x80, 0x01] (continuation, then 1).
            let mut buf = Vec::new();
            write_var_uint(&mut buf, 128);
            assert_eq!(buf, vec![0x80, 0x01]);
        }
    }
}

// ---------------------------------------------------------------------------
// State vector
// ---------------------------------------------------------------------------

/// A Yjs `Y.StateVector` — a `clientId → clock` map describing which ops
/// each client has produced through which counter.
///
/// In Yjs this maps `client: number` to `clock: number`. We treat both
/// as `u64` to match `OpId.replica` and `OpId.counter`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StateVector {
    /// `replica → highest seen counter`, byte-identical to a Yjs
    /// `Map<number, number>` modulo wire encoding.
    pub clocks: BTreeMap<ReplicaId, u64>,
}

impl StateVector {
    /// Empty state vector — equivalent to "I have seen nothing".
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct clients in the vector.
    #[must_use]
    pub fn len(&self) -> usize {
        self.clocks.len()
    }

    /// Is the vector empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clocks.is_empty()
    }

    /// Highest counter known for `replica`, or 0 if none.
    #[must_use]
    pub fn get(&self, replica: ReplicaId) -> u64 {
        self.clocks.get(&replica).copied().unwrap_or(0)
    }

    /// Encode in Y.Doc's `encodeStateVector` byte format. Identical to
    /// what `Y.encodeStateVector(doc)` produces in the JS Yjs library.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.clocks.len() * 4);
        lib0::write_var_uint(&mut buf, self.clocks.len() as u64);
        for (&replica, &clock) in &self.clocks {
            lib0::write_var_uint(&mut buf, replica);
            lib0::write_var_uint(&mut buf, clock);
        }
        buf
    }

    /// Decode bytes produced by `Y.encodeStateVector(doc)` (or by
    /// [`Self::encode`]).
    ///
    /// # Errors
    ///
    /// Returns a [`lib0::Error`] if the buffer is truncated or contains
    /// values that don't fit in `u64`.
    pub fn decode(bytes: &[u8]) -> Result<Self, lib0::Error> {
        let mut cursor = 0usize;
        let (count, n) = lib0::read_var_uint(&bytes[cursor..])?;
        cursor += n;
        let mut clocks = BTreeMap::new();
        for _ in 0..count {
            let (replica, n) = lib0::read_var_uint(&bytes[cursor..])?;
            cursor += n;
            let (clock, n) = lib0::read_var_uint(&bytes[cursor..])?;
            cursor += n;
            clocks.insert(replica, clock);
        }
        Ok(Self { clocks })
    }

    /// Convert from this crate's [`VersionVector`].
    #[must_use]
    pub fn from_version(v: &VersionVector) -> Self {
        let mut clocks = BTreeMap::new();
        // VersionVector doesn't expose its inner map directly; iterate the
        // counters via get(replica) for replicas we know about. Cheaper
        // path: we assume v has at most a handful of replicas.
        //
        // For now: just clone via Display/Debug → no, use serde.
        //
        // Cleanest: provide a Vec<(replica, counter)> accessor on VersionVector.
        // We expose .iter() via this conversion path.
        for (replica, counter) in v.iter_clocks() {
            clocks.insert(replica, counter);
        }
        Self { clocks }
    }

    /// Convert to this crate's [`VersionVector`].
    #[must_use]
    pub fn to_version(&self) -> VersionVector {
        let mut v = VersionVector::new();
        for (&replica, &counter) in &self.clocks {
            // VersionVector's observe is for a specific OpId. We need a way
            // to set "highest counter" directly. Expose via observe of
            // (counter, replica).
            v.observe(crate::id::OpId::new(counter, replica));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_vector_round_trip() {
        let sv = StateVector::new();
        let bytes = sv.encode();
        let restored = StateVector::decode(&bytes).unwrap();
        assert_eq!(sv, restored);
    }

    #[test]
    fn state_vector_round_trip() {
        let mut sv = StateVector::new();
        sv.clocks.insert(1, 100);
        sv.clocks.insert(2, 50);
        sv.clocks.insert(99, 1_000_000);
        let bytes = sv.encode();
        let restored = StateVector::decode(&bytes).unwrap();
        assert_eq!(sv, restored);
    }

    #[test]
    fn state_vector_yjs_byte_format() {
        // Yjs would encode {1: 5, 2: 7} as:
        //   [count=2, client=1, clock=5, client=2, clock=7]
        // = [0x02, 0x01, 0x05, 0x02, 0x07]
        let mut sv = StateVector::new();
        sv.clocks.insert(1, 5);
        sv.clocks.insert(2, 7);
        assert_eq!(sv.encode(), vec![0x02, 0x01, 0x05, 0x02, 0x07]);
    }

    #[test]
    fn convert_to_from_version_vector() {
        let mut sv = StateVector::new();
        sv.clocks.insert(7, 42);
        sv.clocks.insert(8, 11);
        let v = sv.to_version();
        let sv2 = StateVector::from_version(&v);
        assert_eq!(sv, sv2);
    }
}
