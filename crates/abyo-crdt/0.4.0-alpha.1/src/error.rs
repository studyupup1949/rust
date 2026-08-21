//! Error types.

use thiserror::Error;

/// Errors that can occur in `abyo-crdt`.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Position is out of bounds for the current visible length.
    #[error("position {pos} is out of bounds (visible length {len})")]
    OutOfBounds {
        /// The requested position.
        pos: usize,
        /// The current visible length.
        len: usize,
    },

    /// A remote insertion referenced a parent that we have not seen.
    ///
    /// In a correct event-graph delivery, parents are always delivered
    /// before children. This error indicates the caller violated causal
    /// delivery (e.g. by applying `Op`s out of order).
    #[error("missing causal parent {missing:?} for op {op:?}")]
    MissingParent {
        /// The op being applied.
        op: crate::OpId,
        /// The parent that wasn't found.
        missing: crate::OpId,
    },

    /// A delete op targets an item we have not seen.
    #[error("delete op {op:?} targets unknown item {target:?}")]
    UnknownTarget {
        /// The delete op.
        op: crate::OpId,
        /// The missing target.
        target: crate::OpId,
    },

    /// A remote op claims to come from this replica's id. Two replicas
    /// must never share an id — if they do, every guarantee is off.
    #[error(
        "remote op {op:?} uses our replica id {replica}; \
         two replicas must never share a replica id"
    )]
    ReplicaIdConflict {
        /// The offending op.
        op: crate::OpId,
        /// The shared replica id.
        replica: crate::ReplicaId,
    },

    /// The Lamport clock would overflow `u64`. Practically never reachable
    /// (~10¹⁹ ops per replica), but reported instead of silently wrapping.
    #[error("Lamport clock overflow on replica {replica}")]
    ClockOverflow {
        /// The replica that overflowed.
        replica: crate::ReplicaId,
    },
}
