//! Streaming JSON path extractor.
//!
//! Register [RFC 6901 JSON Pointer] paths before parsing begins; feed byte
//! chunks as they arrive; receive matching scalar values via callback —
//! without buffering the whole document.
//!
//! [RFC 6901 JSON Pointer]: https://www.rfc-editor.org/rfc/rfc6901
//!
//! # Quick start
//!
//! ```rust
//! use acutejson::{Builder, Status};
//!
//! let mut parser = Builder::new()
//!     .register("/user/name", |bytes, is_complete| {
//!         if is_complete {
//!             println!("name: {}", std::str::from_utf8(bytes).unwrap());
//!         }
//!     })
//!     .unwrap()
//!     .register("/user/age", |bytes, is_complete| {
//!         if is_complete {
//!             println!("age: {}", std::str::from_utf8(bytes).unwrap());
//!         }
//!     })
//!     .unwrap()
//!     .build();
//!
//! let chunk = b"{\"user\":{\"name\":\"Alice\",\"age\":30}}";
//! match parser.feed(chunk).unwrap() {
//!     Status::Done    => {} // all registered paths resolved — can stop early
//!     Status::NeedMore => {} // keep feeding
//! }
//! parser.finish().unwrap();
//! ```
//!
//! # Callback contract
//!
//! Each registered callback receives raw JSON bytes as they arrive:
//!
//! - **Strings** — streamed directly from the input buffer. The callback may
//!   fire multiple times (once per chunk); `is_complete = true` on the call
//!   that follows the closing `"`.  Raw escape sequences (e.g. `\n`, `A`)
//!   are forwarded as-is; the caller is responsible for decoding them.
//! - **Numbers, booleans, `null`** — buffered internally and delivered in a
//!   single call with `is_complete = true`.
//!
//! # Limitations
//!
//! - Only the **first occurrence** of each path is matched.
//! - Paths pointing to a **container** (object or array) receive no callback;
//!   only scalar values (string, number, boolean, null) are delivered.

mod parser;
mod trie;

pub use parser::{ParseError, Parser, Status};
pub use trie::{Builder, PointerError};
