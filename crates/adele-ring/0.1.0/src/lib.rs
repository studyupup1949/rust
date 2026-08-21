//! # adele-ring — exact multi-base arithmetic engine
//!
//! `adele-ring` represents numbers across multiple prime channels using the
//! Residue Number System (RNS). Because the channels are mutually independent
//! (no carry propagation), arithmetic is embarrassingly parallel and maps onto
//! both CPU threads (rayon) and GPU threads (wgpu).
//!
//! On top of the RNS substrate sits a *number tower* that keeps every value at
//! the cheapest exact level it can:
//!
//! | Level | Set      | Type                         |
//! |-------|----------|------------------------------|
//! | 0     | ℤ        | [`rns::RnsInt`]              |
//! | 1     | ℚ        | [`rational::RnsRational`]    |
//! | 2     | ℚ̄        | [`algebraic::AlgebraicNumber`] |
//! | 3     | ℝ_c      | [`computable::ComputableReal`] |
//! | 4     | 𝒮        | [`symbolic::SymbolicExpr`]   |
//!
//! The crate name uses a hyphen (`adele-ring`) but the Rust path uses an
//! underscore (`adele_ring`), per Rust's identifier rules.

/// Channel-count threshold above which single-value operations parallelize over
/// channels with rayon. Below it, sequential is faster (task overhead ~50ns vs
/// channel op ~1ns: break-even around 8–16 channels).
pub const RAYON_CHANNEL_THRESHOLD: usize = 16;

// ── Phase 1: parallelism foundation ──────────────────────────────────────────
pub mod backend;
pub mod batch;
pub mod cpu;
pub mod gpu;
pub mod primes;
pub mod rns;

// ── Phase 2: the number tower ────────────────────────────────────────────────
// (enabled incrementally as each module lands)
pub mod algebraic;
pub mod computable;
pub mod dispatch;
pub mod rational;
pub mod symbolic;
pub mod tower;

pub use algebraic::{AlgebraicNumber, Polynomial};
pub use backend::{executor, ArithmeticBackend, Executor};
pub use batch::RnsBatch;
pub use computable::{Computable, ComputableReal};
pub use dispatch::{DispatchPlan, Dispatcher};
pub use rational::RnsRational;
pub use rns::{garner_crt, Channels, RnsInt};
pub use symbolic::{IdentityGraph, SymbolicExpr};
pub use tower::{TowerLevel, TowerValue};
