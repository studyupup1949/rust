//! Adaptive timeout computation based on observed latency quantiles.
//!
//! A [`LatencyTracker`] records latencies per destination using sliding-window
//! histograms. An [`AdaptiveTimeout`] queries the tracker for a high quantile,
//! applies a safety factor and exponential backoff, and clamps the result
//! between a configurable floor and ceiling.
//!
//! When insufficient data is available, falls back to pure exponential backoff.
//!
//! # Per-service tracking
//!
//! Create one [`LatencyTracker`] per service/operation type. This keeps each
//! service's latency distribution independent.
//!
//! # Example
//!
//! ```rust
//! use std::time::{Duration, Instant};
//! use adaptive_timeout::{AdaptiveTimeout, LatencyTracker};
//!
//! let now = Instant::now();
//! let mut tracker = LatencyTracker::<u32, Instant>::default();
//! let timeout = AdaptiveTimeout::default();
//!
//! // No data yet — falls back to exponential backoff.
//! let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
//! assert_eq!(t, Duration::from_millis(250));
//!
//! // Record some latency observations.
//! for _ in 0..100 {
//!     tracker.record_latency(&1u32, Duration::from_millis(50), now);
//! }
//!
//! // Now the timeout adapts to observed latencies.
//! let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
//! assert!(t >= Duration::from_millis(50), "timeout should reflect observed latency");
//! ```

pub mod clock;
mod config;
mod histogram;
mod parse;
#[cfg(feature = "sync")]
mod sync_tracker;
mod timeout;
mod tracker;

pub use clock::Instant;
pub use config::{MillisNonZero, TimeoutConfig, TrackerConfig};
pub use parse::{BackoffInterval, ParseError};
#[cfg(feature = "sync")]
pub use sync_tracker::SyncLatencyTracker;
pub use timeout::AdaptiveTimeout;
pub use tracker::DEFAULT_SUB_WINDOWS;
pub use tracker::LatencyTracker;
