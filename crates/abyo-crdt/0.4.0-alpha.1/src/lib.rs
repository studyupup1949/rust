//! # abyo-crdt
//!
//! Pure Rust CRDT library implementing an [Eg-walker]-style event log over a
//! [Fugue-Maximal] list, with companion [`Map`], [`Counter`], and [`Set`]
//! CRDTs and [Peritext] rich text planned for v0.3.
//!
//! See the [`README`](https://github.com/abyo-software/abyo-crdt) for design
//! background and the [crate-level plan](https://github.com/abyo-software/abyo-crdt/blob/main/abyo_crdt_plan.md)
//! for the roadmap.
//!
//! [Eg-walker]: https://arxiv.org/abs/2409.14252
//! [Fugue-Maximal]: https://arxiv.org/abs/2305.00583
//! [Peritext]: https://www.inkandswitch.com/peritext/
//!
//! ## Available data types
//!
//! | Type           | Algorithm        | Status     |
//! |----------------|------------------|------------|
//! | [`List<T>`]    | Fugue-Maximal    | v0.1 ✅    |
//! | [`Map<K, V>`]  | LWW with Lamport | v0.2 ✅    |
//! | [`Counter`]    | PN-Counter       | v0.2 ✅    |
//! | [`Set<T>`]     | OR-Set, add-wins | v0.2 ✅    |
//! | `Text`         | Peritext         | v0.3 🚧    |
//!
//! Every CRDT in the crate ships an event log (`ops()`), supports incremental
//! sync via [`VersionVector`] (`ops_since(&version)`), and is `Serialize +
//! Deserialize` under the default `serde` feature.
//!
//! ## Quick start
//!
//! ```
//! use abyo_crdt::List;
//!
//! let mut alice = List::<char>::new(1);
//! alice.insert(0, 'H');
//! alice.insert(1, 'i');
//!
//! let mut bob = List::<char>::new(2);
//! bob.merge(&alice);
//!
//! bob.insert(2, '!');
//! alice.merge(&bob);
//!
//! assert_eq!(alice.to_vec(), vec!['H', 'i', '!']);
//! assert_eq!(bob.to_vec(), vec!['H', 'i', '!']);
//! ```
//!
//! ## Concurrent edits never interleave
//!
//! Fugue-Maximal guarantees that contiguous bursts of typing stay contiguous
//! after merge, regardless of timing.
//!
//! ```
//! use abyo_crdt::List;
//!
//! let mut alice = List::<char>::new(1);
//! let mut bob = List::<char>::new(2);
//!
//! // Both start from a shared "ab" doc
//! alice.insert(0, 'a');
//! alice.insert(1, 'b');
//! bob.merge(&alice);
//!
//! // Concurrent inserts at position 1 — Alice types "Hello", Bob types "World"
//! for (i, c) in "Hello".chars().enumerate() {
//!     alice.insert(1 + i, c);
//! }
//! for (i, c) in "World".chars().enumerate() {
//!     bob.insert(1 + i, c);
//! }
//!
//! alice.merge(&bob);
//! bob.merge(&alice);
//!
//! let merged: String = alice.iter().collect();
//! // Either "aHelloWorldb" or "aWorldHellob" — never interleaved.
//! assert!(merged == "aHelloWorldb" || merged == "aWorldHellob");
//! assert_eq!(merged, bob.iter().collect::<String>());
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

mod counter;
mod cursor;
mod error;
mod id;
mod list;
mod map;
mod ost;
mod set;
#[cfg(feature = "storage")]
pub mod storage;
mod text;
mod version;
pub mod yjs_compat;

pub use counter::{Counter, CounterOp};
pub use cursor::{Cursor, CursorSide, Selection};
pub use error::Error;
pub use id::{new_replica_id, OpId, ReplicaId};
pub use list::{List, ListOp, Side};
pub use map::{Map, MapOp};
pub use set::{Set, SetOp};
pub use text::{
    Anchor, AnchorSide, AttrValue, DeltaOp, ExpandRule, MarkSet, MarkValue, Span, SpanValue, Text,
    TextOp,
};
pub use version::VersionVector;
