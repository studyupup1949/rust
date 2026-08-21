//! Minimal **Y.Update v1** encoder — convert an abyo-crdt [`Text`]
//! snapshot to a binary that `Y.Doc.applyUpdate` accepts.
//!
//! ## Scope
//!
//! This is a **one-way snapshot** translator, not a full Yjs sync
//! protocol. It produces an update message containing a single
//! `Y.Text` typed at root key `"abyo"` whose content matches the
//! source `Text`'s visible characters. Format marks are NOT
//! translated (Yjs's format-attribute model is sufficiently
//! different that round-trip is lossy; use [`crate::Text::to_delta`]
//! for marks).
//!
//! Use this for the common "Rust server has the canonical document,
//! browser running Yjs needs to bootstrap" handoff.
//!
//! ## Wire format
//!
//! Y.Update v1 is:
//!
//! ```text
//! [num_clients : var-uint]
//!   for each client:
//!     [client_id : var-uint]
//!     [num_structs : var-uint]
//!     for each struct:
//!       [info byte] [origin? : ID] [right_origin? : ID]
//!       [parent_info? : ParentInfo] [parent_sub? : var-string]
//!       [content : type-tagged]
//! [delete_set : var-uint num_clients
//!   for each client:
//!     [client_id : var-uint] [num_ranges : var-uint]
//!     for each range: [clock : var-uint] [len : var-uint]]
//! ```
//!
//! For our snapshot encoder: 1 client, N structs (one per char),
//! empty delete set.

use crate::yjs_compat::lib0;
use crate::Text;

/// Magic name we register the Y.Text at in the produced Y.Doc.
const TEXT_TYPE_NAME: &str = "abyo";

/// Yjs item info-byte bits (from `lib0/encoding.js`).
const INFO_KEEP: u8 = 0;
const INFO_HAS_ORIGIN: u8 = 0b1000_0000;
#[allow(dead_code)]
const INFO_HAS_RIGHT_ORIGIN: u8 = 0b0100_0000;
const INFO_HAS_PARENT_INFO: u8 = 0b0010_0000;
#[allow(dead_code)]
const INFO_HAS_PARENT_SUB: u8 = 0b0001_0000;

/// Yjs content-type tags.
const CONTENT_STRING: u8 = 4;

/// Encode `text`'s current visible content as a Yjs Y.Update v1 binary.
/// The produced bytes can be fed to `Y.applyUpdateV1(doc, bytes)` in
/// a Yjs browser, yielding a `Y.Text` at root key
/// [`TEXT_TYPE_NAME`] (`"abyo"`) with the same characters.
///
/// **Lossy**: format marks are not transferred (use
/// [`crate::Text::to_delta`] for marks). The Yjs IDs do not match the
/// source `OpId`s — Yjs sees a single-author chain.
#[must_use]
pub fn snapshot_text_to_yjs_update(text: &Text) -> Vec<u8> {
    let s: String = text.iter_with_marks().map(|(c, _)| c).collect();
    snapshot_string_to_yjs_update(&s, text.replica_id())
}

/// Lower-level: encode an arbitrary `&str` as a Y.Update with a single
/// `Y.Text` at the root, attributed to `client_id`.
#[must_use]
pub fn snapshot_string_to_yjs_update(s: &str, client_id: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 4 + 32);

    // We emit one struct per UTF-8 codepoint, chained via origin pointers.
    // The first struct has no origin; subsequent structs have origin =
    // (client, prev clock).
    let chars: Vec<char> = s.chars().collect();
    let num_clients: u64 = u64::from(!chars.is_empty());

    lib0::write_var_uint(&mut out, num_clients);

    if !chars.is_empty() {
        // Client header.
        lib0::write_var_uint(&mut out, client_id);
        lib0::write_var_uint(&mut out, chars.len() as u64);

        for (i, ch) in chars.iter().enumerate() {
            // Info byte for this struct:
            //   - has_origin if i > 0
            //   - has_parent_info on the FIRST struct (defines the type)
            //   - content type = CONTENT_STRING
            let mut info = INFO_KEEP | CONTENT_STRING;
            if i > 0 {
                info |= INFO_HAS_ORIGIN;
            } else {
                info |= INFO_HAS_PARENT_INFO;
            }
            out.push(info);

            // Origin: ID of the previous struct (= same client, clock = i - 1).
            if i > 0 {
                lib0::write_var_uint(&mut out, client_id);
                lib0::write_var_uint(&mut out, (i - 1) as u64);
            }

            // No right-origin in this minimal encoding (we always append).

            // Parent info: only on the FIRST struct. Encodes "this Y.Text
            // is registered at root key TEXT_TYPE_NAME on the Y.Doc."
            if i == 0 {
                // ParentInfo format: 1 = string parent, then var-string.
                out.push(1u8);
                lib0::write_var_string(&mut out, TEXT_TYPE_NAME);
            }

            // Content: STRING type, length-prefixed UTF-8 of just this char.
            // (Real Yjs runs of inserts get coalesced into a single string
            // content to save bytes; we emit one char per struct for
            // simplicity.)
            let mut buf = [0u8; 4];
            let ch_str = ch.encode_utf8(&mut buf);
            lib0::write_var_string(&mut out, ch_str);
        }
    }

    // Delete set: empty (no client has deletions in the snapshot).
    lib0::write_var_uint(&mut out, 0);

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_encodes_to_empty_update() {
        let text = Text::new(1);
        let bytes = snapshot_text_to_yjs_update(&text);
        // Should be: [num_clients=0, delete_set_clients=0] = [0, 0]
        assert_eq!(bytes, vec![0, 0]);
    }

    #[test]
    fn single_char_encoding_smoke() {
        // "a" (1 char). Expected structure:
        //   num_clients = 1
        //   client header: client_id, num_structs=1
        //   info byte: 0b00100100 = HAS_PARENT_INFO | CONTENT_STRING
        //   parent_info: kind=1, var-string "abyo"
        //   content: var-string "a"
        // delete_set: 0
        let bytes = snapshot_string_to_yjs_update("a", 42);
        assert_eq!(bytes[0], 1, "num_clients");
        assert_eq!(bytes[1], 42, "client_id");
        assert_eq!(bytes[2], 1, "num_structs");
        // info byte
        assert_eq!(bytes[3], INFO_HAS_PARENT_INFO | CONTENT_STRING, "info byte");
        // parent kind = 1
        assert_eq!(bytes[4], 1);
        // var-string "abyo": [4, 'a','b','y','o']
        assert_eq!(&bytes[5..10], &[4, b'a', b'b', b'y', b'o']);
        // content var-string "a": [1, 'a']
        assert_eq!(&bytes[10..12], &[1, b'a']);
        // delete set: 0
        assert_eq!(bytes[12], 0);
        assert_eq!(bytes.len(), 13);
    }

    #[test]
    fn multi_char_chain() {
        let bytes = snapshot_string_to_yjs_update("abc", 1);
        // First char has parent info; subsequent have origin.
        // Just verify structure round-trip is non-empty and reasonable size.
        assert!(bytes.len() > 10);
        // num_clients=1, num_structs=3
        assert_eq!(bytes[0], 1);
        assert_eq!(bytes[1], 1); // client_id = 1
        assert_eq!(bytes[2], 3); // num_structs = 3
    }

    #[test]
    fn snapshot_text_uses_replica_id_as_client() {
        let mut text = Text::new(7);
        text.insert_str(0, "hi");
        let bytes = snapshot_text_to_yjs_update(&text);
        // [num_clients=1, client_id=7, num_structs=2, ...]
        assert_eq!(bytes[0], 1);
        assert_eq!(bytes[1], 7);
        assert_eq!(bytes[2], 2);
    }
}
