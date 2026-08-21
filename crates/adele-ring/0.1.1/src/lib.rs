//! # adele-ring — exact multi-base arithmetic engine
//!
//! `adele-ring` carries every number at **two kinds of place at once**. The
//! *finite places* (an adaptive residue-number system over a prime [`Basis`])
//! do all the real arithmetic: carry-free, local, embarrassingly parallel across
//! CPU lanes and GPU threads, and free of big-integer work. The *infinite place*
//! (a rigorous real interval, [`Ball`]) answers every question the prime channels
//! constitutionally cannot — sign, comparison, magnitude, and decimal output — by
//! refining on demand. Exactness is recovered by reconstructing from the finite
//! places, with the basis grown to exactly the height the computation provably
//! needs ([`bounds`]), so a result can never silently exceed its range. Big
//! integers appear only at that reconstruction boundary, for an instant, on
//! numbers as large as the answer's own information content — the product formula
//! reminding us that what we save among the primes we pay, once, at infinity.
//!
//! This pairing is the [`Adelic`] carrier, the data-structure form of
//! `𝔸_ℚ = ℝ × ∏′_p ℚ_p`. On top of the RNS substrate sits a *number tower* that
//! keeps every value at the cheapest exact level it can.
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

// ── Foundation: the adelic carrier (finite places × the real place) ──────────
pub mod adelic;
pub mod ball;
pub mod basis;
pub mod bounds;
pub mod error;
pub mod reconstruct;

// ── Parallelism foundation ───────────────────────────────────────────────────
pub mod backend;
pub mod batch;
pub mod cpu;
pub mod gpu;
pub mod primes;
pub mod rns;

// ── The number tower ─────────────────────────────────────────────────────────
pub mod algebraic;
pub mod computable;
pub mod dispatch;
pub mod rational;
pub mod symbolic;
pub mod tower;

pub use adelic::{Adelic, AdelicInt, AdelicRat, Finite, RnsFrac};
pub use algebraic::{AlgebraicNumber, Polynomial};
pub use backend::{executor, ArithmeticBackend, Executor};
pub use ball::Ball;
pub use basis::Basis;
pub use batch::RnsBatch;
pub use bounds::{determinant_bits, mignotte_bits, resultant_bits};
pub use computable::{Computable, ComputableReal};
pub use dispatch::{DispatchPlan, Dispatcher};
pub use error::{BasisError, ChannelMismatch, GpuError, RangeError};
pub use rational::RnsRational;
pub use reconstruct::rational_reconstruct;
pub use rns::{crt_balanced, garner_crt, RnsInt};
pub use symbolic::{IdentityGraph, SymbolicExpr};
pub use tower::{TowerLevel, TowerValue};
