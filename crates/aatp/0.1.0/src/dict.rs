//! Adaptive payload compression via a **compile-time dictionary** of the tokens
//! agents repeat — tool names, roles, stock phrases. Each dictionary entry gets a
//! one-byte code, so a 12-byte `"orchestrator"` collapses to two bytes on the wire.
//!
//! The dictionary is a `const`: swap [`DICTIONARY`] for *your* fleet's vocabulary and
//! recompile, and every payload shrinks by exactly those tokens — the protocol adapts
//! to your agents' "language" with zero runtime cost and no added code complexity.
//!
//! Encoding: a dictionary hit is `ESCAPE, code` where `code = index + 1`; a literal
//! `ESCAPE` byte in the data is `ESCAPE, 0`. Everything else is copied verbatim, so
//! the transform is exactly reversible — [`decompress`]∘[`compress`] is the identity.

/// The escape byte introducing a code. `0x1B` (ASCII ESC) is rare in agent text, so
/// literal-escape stuffing almost never fires.
pub const ESCAPE: u8 = 0x1B;

/// The self-optimizing dictionary — the tokens your agents say most. Replace with your
/// own and recompile. Ordered longest-first so the greedy match takes the biggest bite.
/// At most 255 entries (codes `1..=255`; code `0` means a literal `ESCAPE`).
pub const DICTIONARY: &[&str] = &[
    "orchestrator",
    "web_search",
    "code_exec",
    "file_read",
    "assistant",
    "planner",
    "system",
    "tool",
    "user",
];

/// What can go wrong compressing or decompressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictError {
    /// The output buffer is too small for the result.
    OutputTooSmall,
    /// A trailing `ESCAPE` with no code byte after it.
    TruncatedEscape,
    /// An `ESCAPE, code` whose `code` names no dictionary entry.
    BadCode(u8),
}

/// The index of the first dictionary entry that is a prefix of `data[at..]`, or
/// `None`. Longest-first ordering in [`DICTIONARY`] makes "first" the greedy choice.
fn dict_match(data: &[u8], at: usize) -> Option<usize> {
    let rest = &data[at..];
    let mut k = 0;
    while k < DICTIONARY.len() {
        if rest.starts_with(DICTIONARY[k].as_bytes()) {
            return Some(k);
        }
        k += 1;
    }
    None
}

/// Compress `data` into `out`, returning the compressed length. Worst case (every byte
/// is `ESCAPE`) is `2 * data.len()`; size `out` accordingly.
pub fn compress(data: &[u8], out: &mut [u8]) -> Result<usize, DictError> {
    let mut i = 0;
    let mut o = 0;
    while i < data.len() {
        if let Some(k) = dict_match(data, i) {
            put(out, &mut o, ESCAPE)?;
            put(out, &mut o, (k as u8) + 1)?; // code 1..=len, never 0
            i += DICTIONARY[k].len();
        } else if data[i] == ESCAPE {
            put(out, &mut o, ESCAPE)?;
            put(out, &mut o, 0)?; // 0 = literal escape
            i += 1;
        } else {
            put(out, &mut o, data[i])?;
            i += 1;
        }
    }
    Ok(o)
}

/// Decompress `data` into `out`, returning the original length. The exact inverse of
/// [`compress`].
pub fn decompress(data: &[u8], out: &mut [u8]) -> Result<usize, DictError> {
    let mut i = 0;
    let mut o = 0;
    while i < data.len() {
        if data[i] == ESCAPE {
            let code = *data.get(i + 1).ok_or(DictError::TruncatedEscape)?;
            if code == 0 {
                put(out, &mut o, ESCAPE)?; // literal escape
            } else {
                let entry = DICTIONARY
                    .get((code - 1) as usize)
                    .ok_or(DictError::BadCode(code))?;
                put_slice(out, &mut o, entry.as_bytes())?;
            }
            i += 2;
        } else {
            put(out, &mut o, data[i])?;
            i += 1;
        }
    }
    Ok(o)
}

fn put(out: &mut [u8], o: &mut usize, byte: u8) -> Result<(), DictError> {
    let slot = out.get_mut(*o).ok_or(DictError::OutputTooSmall)?;
    *slot = byte;
    *o += 1;
    Ok(())
}

fn put_slice(out: &mut [u8], o: &mut usize, bytes: &[u8]) -> Result<(), DictError> {
    let end = *o + bytes.len();
    let slot = out.get_mut(*o..end).ok_or(DictError::OutputTooSmall)?;
    slot.copy_from_slice(bytes);
    *o = end;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(data: &[u8]) -> std::vec::Vec<u8> {
        let mut packed = std::vec![0u8; data.len() * 2 + 4];
        let n = compress(data, &mut packed).unwrap();
        packed.truncate(n);
        let mut back = std::vec![0u8; data.len() + 4];
        let m = decompress(&packed, &mut back).unwrap();
        back.truncate(m);
        back
    }

    #[test]
    fn round_trip_is_the_identity_over_varied_inputs() {
        for data in [
            &b""[..],
            &b"plain text with no dictionary words at all"[..],
            &b"the orchestrator called web_search then code_exec"[..],
            &b"user and assistant and tool and system"[..],
            &[ESCAPE][..],                       // a bare literal escape
            &[ESCAPE, ESCAPE, b'x', ESCAPE][..], // escapes around data
            &b"web_searchweb_search"[..],        // adjacent dictionary hits
        ] {
            assert_eq!(round_trip(data), data, "round trip must reproduce {data:?}");
        }
    }

    #[test]
    fn a_dictionary_word_compresses_to_two_bytes() {
        let mut out = [0u8; 32];
        let n = compress(b"orchestrator", &mut out).unwrap();
        assert_eq!(n, 2, "a 12-byte token must become a 2-byte code");
        assert_eq!(out[0], ESCAPE);
        assert_eq!(out[1], 1); // "orchestrator" is index 0 → code 1
    }

    #[test]
    fn a_literal_escape_byte_is_stuffed_and_recovered() {
        let mut out = [0u8; 8];
        let n = compress(&[ESCAPE], &mut out).unwrap();
        assert_eq!(&out[..n], &[ESCAPE, 0]); // ESCAPE,0 = a literal escape
        let mut back = [0u8; 8];
        let m = decompress(&out[..n], &mut back).unwrap();
        assert_eq!(&back[..m], &[ESCAPE]);
    }

    #[test]
    fn dict_match_finds_a_prefix_only_at_the_current_position() {
        assert_eq!(dict_match(b"tool", 0), Some(7)); // "tool" is index 7
        assert_eq!(dict_match(b"xtool", 0), None); // not a prefix at 0
        assert_eq!(dict_match(b"xtool", 1), Some(7)); // is a prefix at 1
        assert_eq!(dict_match(b"too", 0), None); // partial word is not a hit
    }

    #[test]
    fn compress_rejects_a_too_small_output_at_the_boundary() {
        // "orchestrator" needs 2 bytes; one byte is not enough.
        let mut one = [0u8; 1];
        assert_eq!(
            compress(b"orchestrator", &mut one),
            Err(DictError::OutputTooSmall)
        );
        let mut two = [0u8; 2];
        assert_eq!(compress(b"orchestrator", &mut two), Ok(2));
    }

    #[test]
    fn decompress_rejects_a_truncated_escape() {
        assert_eq!(
            decompress(&[ESCAPE], &mut [0u8; 8]),
            Err(DictError::TruncatedEscape)
        );
    }

    #[test]
    fn decompress_rejects_a_code_past_the_dictionary() {
        let bad = DICTIONARY.len() as u8 + 1; // one past the last valid code
        assert_eq!(
            decompress(&[ESCAPE, bad], &mut [0u8; 32]),
            Err(DictError::BadCode(bad))
        );
    }

    #[test]
    fn decompress_rejects_a_too_small_output_for_an_expanded_token() {
        // ESCAPE,1 expands to "orchestrator" (12 bytes) — 4 bytes of room is not enough.
        assert_eq!(
            decompress(&[ESCAPE, 1], &mut [0u8; 4]),
            Err(DictError::OutputTooSmall)
        );
    }

    #[test]
    fn dictionary_invariants_hold_for_the_shipped_and_any_edited_vocabulary() {
        // The dictionary is editable, so pin its two safety invariants here (a failing
        // build, not a silent hazard, if either breaks):
        //  - at most 255 entries: a 256th would overflow the one-byte code `(k as u8)+1`;
        assert!(
            DICTIONARY.len() <= 255,
            "at most 255 entries (one-byte codes)"
        );
        assert!(!DICTIONARY.is_empty());
        //  - no empty entry: an empty prefix matches everywhere, so `compress` would
        //    advance by zero (`i += 0`) and loop forever.
        for entry in DICTIONARY {
            assert!(!entry.is_empty(), "a dictionary entry must be non-empty");
        }
    }

    #[test]
    fn shipped_dictionary_is_pinned_entry_for_entry() {
        // A SNAPSHOT of the shipped vocabulary. `compress`/`decompress` round-trip under
        // ANY dictionary (they read the same one), so the round-trip tests cannot catch a
        // typo'd entry — this can. When you intentionally re-vocabulary the dictionary for
        // your fleet, update this snapshot; an *accidental* edit reddens it in review.
        assert_eq!(
            DICTIONARY,
            &[
                "orchestrator",
                "web_search",
                "code_exec",
                "file_read",
                "assistant",
                "planner",
                "system",
                "tool",
                "user",
            ]
        );
    }
}
