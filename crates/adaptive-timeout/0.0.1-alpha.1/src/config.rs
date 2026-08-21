use std::num::NonZeroU32;
use std::time::Duration;

use crate::BackoffInterval;

/// Milliseconds stored as a [`NonZeroU32`].
///
/// Compact (4-byte) representation for timeout durations that are always
/// positive. Max representable value is ~49.7 days.
pub type MillisNonZero = NonZeroU32;

/// Creates a [`MillisNonZero`] from a `u32` literal at compile time.
///
/// # Panics
///
/// Panics if `ms == 0`.
#[inline]
pub(crate) const fn millis(ms: u32) -> MillisNonZero {
    match NonZeroU32::new(ms) {
        Some(v) => v,
        None => panic!("millis value must be non-zero"),
    }
}

/// HdrHistogram precision used internally. 2 significant digits gives ~1%
/// accuracy, which is more than sufficient for latency tracking.
pub(crate) const SIGNIFICANT_VALUE_DIGITS: u8 = 2;

/// Configuration for [`LatencyTracker`](crate::LatencyTracker).
///
/// All duration-like fields use integer milliseconds for compactness.
///
/// # Example
///
/// ```
/// use adaptive_timeout::TrackerConfig;
///
/// let config = TrackerConfig {
///     min_samples: 10,
///     ..TrackerConfig::default()
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TrackerConfig {
    /// Total sliding window duration in milliseconds. This is the full time
    /// span the histogram remembers — not per sub-window. The window is
    /// divided into `N` equal sub-windows of `window_ms / N` each.
    ///
    /// Default: 60,000 (60s).
    pub window_ms: NonZeroU32,

    /// Minimum samples before quantile estimates are valid. Below this,
    /// queries return `None`. Default: 3.
    pub min_samples: u32,

    /// Maximum trackable latency in milliseconds. Values above this are
    /// clamped. Default: 60,000 (60s).
    pub max_trackable_latency_ms: u32,
}

impl TrackerConfig {
    /// Returns the sliding window duration as a [`Duration`].
    #[inline]
    pub fn window(&self) -> Duration {
        Duration::from_millis(self.window_ms.get() as u64)
    }
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            window_ms: millis(60_000),
            min_samples: 3,
            max_trackable_latency_ms: 60_000,
        }
    }
}

/// Configuration for [`AdaptiveTimeout`](crate::AdaptiveTimeout).
///
/// The struct fits in 24 bytes.
///
/// # Example
///
/// ```
/// use adaptive_timeout::TimeoutConfig;
///
/// let config = TimeoutConfig {
///     quantile: 0.999,
///     safety_factor: 3.0,
///     ..TimeoutConfig::default()
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TimeoutConfig {
    /// Timeout floor and ceiling. Default: `250ms..1min`.
    pub backoff: BackoffInterval,

    /// Quantile of the latency distribution to use (e.g. 0.9999 for P99.99).
    /// Default: 0.9999.
    pub quantile: f64,

    /// Multiplier applied to the quantile estimate. A factor of 2.0 means
    /// the timeout is twice the observed quantile. Default: 2.0.
    pub safety_factor: f64,
}

impl TimeoutConfig {
    /// Returns the minimum timeout as a [`Duration`].
    #[inline]
    pub fn min_timeout(&self) -> Duration {
        Duration::from_millis(self.backoff.min_ms.get() as u64)
    }

    /// Returns the maximum timeout as a [`Duration`].
    #[inline]
    pub fn max_timeout(&self) -> Duration {
        Duration::from_millis(self.backoff.max_ms.get() as u64)
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            backoff: BackoffInterval::default(),
            quantile: 0.9999,
            safety_factor: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn timeout_config_is_compact() {
        assert_eq!(mem::size_of::<TimeoutConfig>(), 24);
    }

    #[test]
    fn tracker_config_is_compact() {
        assert!(mem::size_of::<TrackerConfig>() <= 12);
    }

    #[test]
    fn millis_helper() {
        let m = millis(42);
        assert_eq!(m.get(), 42);
    }

    #[test]
    #[should_panic(expected = "non-zero")]
    fn millis_zero_panics() {
        let _ = millis(0);
    }

    #[test]
    fn timeout_config_conversions() {
        let cfg = TimeoutConfig::default();
        assert_eq!(cfg.min_timeout(), Duration::from_millis(250));
        assert_eq!(cfg.max_timeout(), Duration::from_millis(60_000));
    }

    #[test]
    fn tracker_config_window_conversion() {
        let cfg = TrackerConfig::default();
        assert_eq!(cfg.window(), Duration::from_secs(60));
    }
}
