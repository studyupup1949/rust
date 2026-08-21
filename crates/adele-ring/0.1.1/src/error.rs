//! Crate-wide error types.
//!
//! These make the failure modes the refactor cares about *explicit*: range
//! overflow during reconstruction, basis mismatches, and GPU bring-up failure.

use thiserror::Error;

/// A value could not be reconstructed within the provisioned basis range.
///
/// This is the headline fix from the refactor: exceeding the dynamic range is a
/// **detected event**, never a silent aliasing modulo `M`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RangeError {
    #[error("rational reconstruction failed: have {have_bits} bits of range, need more")]
    ReconstructionFailed { have_bits: u64, need_more: bool },
    #[error("value of {value_bits} bits exceeds basis capacity of {capacity_bits} bits")]
    CapacityExceeded { value_bits: u64, capacity_bits: u64 },
}

/// Problems constructing or combining a [`crate::basis::Basis`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BasisError {
    #[error("requested {requested_bits} bits but no further GPU-eligible primes are available")]
    PrimesExhausted { requested_bits: u64 },
    #[error("operands use different bases (len {a} vs {b})")]
    Mismatch { a: usize, b: usize },
}

/// Operands carried mismatched bases/channels into an operation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("channel/basis mismatch: {0} vs {1}")]
pub struct ChannelMismatch(pub usize, pub usize);

/// GPU backend bring-up failure.
#[derive(Debug, Error)]
pub enum GpuError {
    #[error("no compatible GPU adapter found")]
    NoAdapter,
    #[error("failed to acquire GPU device: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
}
