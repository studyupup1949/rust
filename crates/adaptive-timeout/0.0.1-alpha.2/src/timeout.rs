use std::hash::{BuildHasher, Hash};
use std::time::Duration;

use crate::clock;
use crate::config::TimeoutConfig;
use crate::tracker::LatencyTracker;

/// Computes adaptive timeouts based on observed latency quantiles.
///
/// For each destination, queries the tracker for a high quantile (default:
/// P99.99), applies a safety factor and exponential backoff, clamps between
/// floor and ceiling, and takes the maximum across all destinations.
///
/// Falls back to pure exponential backoff when histogram data is insufficient.
///
/// # Example
///
/// ```
/// use std::time::{Duration, Instant};
/// use adaptive_timeout::{AdaptiveTimeout, LatencyTracker};
///
/// let now = Instant::now();
/// let mut tracker = LatencyTracker::<u32, Instant>::default();
/// let timeout = AdaptiveTimeout::default();
///
/// // No data yet — falls back to exponential backoff (min_timeout).
/// let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
/// assert_eq!(t, Duration::from_millis(250));
/// ```
#[derive(Default, Clone)]
pub struct AdaptiveTimeout {
    config: TimeoutConfig,
}

impl AdaptiveTimeout {
    /// Creates a new `AdaptiveTimeout` with the given configuration.
    pub fn new(config: TimeoutConfig) -> Self {
        Self { config }
    }

    /// Computes an adaptive timeout for a request to the given destinations.
    ///
    /// Returns the maximum timeout across all destinations, clamped to
    /// `[min_timeout, max_timeout]`. `attempt` is 1-based; higher attempts
    /// produce longer timeouts via exponential backoff.
    #[inline]
    pub fn select_timeout<'a, D, I, H, const N: usize>(
        &self,
        tracker: &mut LatencyTracker<D, I, H, N>,
        destinations: impl IntoIterator<Item = &'a D>,
        attempt: u32,
        now: I,
    ) -> Duration
    where
        D: Hash + Eq + Clone + 'a,
        I: clock::Instant,
        H: BuildHasher,
    {
        Duration::from_millis(self.select_timeout_ms(tracker, destinations, attempt, now))
    }

    /// Computes an adaptive timeout in milliseconds.
    /// See [`select_timeout`](Self::select_timeout).
    pub fn select_timeout_ms<'a, D, I, H, const N: usize>(
        &self,
        tracker: &mut LatencyTracker<D, I, H, N>,
        destinations: impl IntoIterator<Item = &'a D>,
        attempt: u32,
        now: I,
    ) -> u64
    where
        D: Hash + Eq + Clone + 'a,
        I: clock::Instant,
        H: BuildHasher,
    {
        let multiplier = Self::attempt_multiplier(attempt);
        let floor = self.config.backoff.min_ms.get() as u64;
        let ceiling = self.config.backoff.max_ms.get() as u64;
        let fallback = (floor * multiplier).min(ceiling);
        let mut selected = fallback;

        for dest in destinations.into_iter() {
            if let Some(estimate_ms) = tracker.quantile_ms(dest, self.config.quantile, now) {
                let adaptive_ms = self.compute_adaptive_ms(estimate_ms, multiplier);
                let clamped = adaptive_ms.max(floor).min(ceiling);
                selected = selected.max(clamped);
            }
        }

        selected
    }

    /// Pure exponential backoff: `min_timeout * 2^(attempt - 1)`, clamped to
    /// `max_timeout`. Fallback when histogram data is insufficient.
    #[inline]
    pub fn exponential_backoff(&self, attempt: u32) -> Duration {
        Duration::from_millis(self.exponential_backoff_ms(attempt))
    }

    /// Pure exponential backoff in milliseconds.
    #[inline]
    pub fn exponential_backoff_ms(&self, attempt: u32) -> u64 {
        let multiplier = Self::attempt_multiplier(attempt);
        let base = self.config.backoff.min_ms.get() as u64;
        let ceiling = self.config.backoff.max_ms.get() as u64;
        (base * multiplier).min(ceiling)
    }

    /// `2^(attempt - 1)`, capped at `2^20`.
    #[inline]
    fn attempt_multiplier(attempt: u32) -> u64 {
        let exponent = attempt.saturating_sub(1).min(20);
        1u64 << exponent
    }

    /// `safety_factor * estimate_ms * multiplier`.
    #[inline]
    fn compute_adaptive_ms(&self, estimate_ms: u64, multiplier: u64) -> u64 {
        let base = estimate_ms.saturating_mul(multiplier);
        (self.config.safety_factor * base as f64) as u64
    }

    /// Returns a reference to the timeout configuration.
    #[inline]
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }

    // -----------------------------------------------------------------------
    // SyncLatencyTracker variants (feature = "sync")
    // -----------------------------------------------------------------------

    /// Like [`select_timeout`](Self::select_timeout) but for
    /// [`SyncLatencyTracker`](crate::SyncLatencyTracker).
    ///
    /// Takes `&tracker` (shared reference) instead of `&mut tracker`.
    #[cfg(feature = "sync")]
    #[inline]
    pub fn select_timeout_sync<'a, D, I, H, const N: usize>(
        &self,
        tracker: &crate::sync_tracker::SyncLatencyTracker<D, I, H, N>,
        destinations: impl IntoIterator<Item = &'a D>,
        attempt: u32,
        now: I,
    ) -> Duration
    where
        D: Hash + Eq + Clone + Send + Sync + 'a,
        I: clock::Instant,
        H: BuildHasher + Clone,
    {
        Duration::from_millis(self.select_timeout_sync_ms(tracker, destinations, attempt, now))
    }

    /// Like [`select_timeout_ms`](Self::select_timeout_ms) but for
    /// [`SyncLatencyTracker`](crate::SyncLatencyTracker).
    #[cfg(feature = "sync")]
    pub fn select_timeout_sync_ms<'a, D, I, H, const N: usize>(
        &self,
        tracker: &crate::sync_tracker::SyncLatencyTracker<D, I, H, N>,
        destinations: impl IntoIterator<Item = &'a D>,
        attempt: u32,
        now: I,
    ) -> u64
    where
        D: Hash + Eq + Clone + Send + Sync + 'a,
        I: clock::Instant,
        H: BuildHasher + Clone,
    {
        let multiplier = Self::attempt_multiplier(attempt);
        let floor = self.config.backoff.min_ms.get() as u64;
        let ceiling = self.config.backoff.max_ms.get() as u64;
        let fallback = (floor * multiplier).min(ceiling);
        let mut selected = fallback;

        for dest in destinations.into_iter() {
            if let Some(estimate_ms) = tracker.quantile_ms(dest, self.config.quantile, now) {
                let adaptive_ms = self.compute_adaptive_ms(estimate_ms, multiplier);
                let clamped = adaptive_ms.max(floor).min(ceiling);
                selected = selected.max(clamped);
            }
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::config::TrackerConfig;
    use crate::parse::BackoffInterval;

    fn make_tracker_and_timeout<I: clock::Instant>() -> (LatencyTracker<u32, I>, AdaptiveTimeout) {
        let tracker_config = TrackerConfig {
            min_samples: 5,
            ..TrackerConfig::default()
        };
        let timeout_config = TimeoutConfig {
            backoff: "10ms..60s".parse::<BackoffInterval>().unwrap(),
            quantile: 0.99,
            safety_factor: 2.0,
        };
        (
            LatencyTracker::new(tracker_config),
            AdaptiveTimeout::new(timeout_config),
        )
    }

    #[test]
    fn fallback_exponential_backoff_no_data() {
        let now = Instant::now();
        let (mut tracker, timeout) = make_tracker_and_timeout();

        let t1 = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
        assert_eq!(t1, Duration::from_millis(10));

        let t2 = timeout.select_timeout(&mut tracker, &[1u32], 2, now);
        assert_eq!(t2, Duration::from_millis(20));

        let t3 = timeout.select_timeout(&mut tracker, &[1u32], 3, now);
        assert_eq!(t3, Duration::from_millis(40));
    }

    #[test]
    fn exponential_backoff_capped_at_max() {
        let now = Instant::now();
        let (mut tracker, timeout) = make_tracker_and_timeout();

        let t = timeout.select_timeout(&mut tracker, &[1u32], 100, now);
        assert_eq!(t, Duration::from_secs(60));
    }

    #[test]
    fn adaptive_timeout_with_data() {
        let now = Instant::now();
        let (mut tracker, timeout) = make_tracker_and_timeout();

        for _ in 0..100 {
            tracker.record_latency(&1u32, Duration::from_millis(50), now);
        }

        // p99 ~50ms, safety_factor=2, attempt=1: 2 * 50 * 1 = 100ms
        let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
        assert_eq!(t, Duration::from_millis(100));
    }

    #[test]
    fn adaptive_timeout_respects_floor() {
        let now = Instant::now();
        let (mut tracker, timeout) = make_tracker_and_timeout();

        for _ in 0..100 {
            tracker.record_latency(&1u32, Duration::from_millis(1), now);
        }

        let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
        assert_eq!(t, Duration::from_millis(10));
    }

    #[test]
    fn adaptive_timeout_respects_ceiling() {
        let now = Instant::now();
        let (mut tracker, timeout) = make_tracker_and_timeout();

        for _ in 0..100 {
            tracker.record_latency(&1u32, Duration::from_millis(50_000), now);
        }

        let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
        assert_eq!(t, Duration::from_secs(60));
    }

    #[test]
    fn max_across_destinations() {
        let now = Instant::now();
        let (mut tracker, timeout) = make_tracker_and_timeout();

        for _ in 0..100 {
            tracker.record_latency(&1u32, Duration::from_millis(10), now);
            tracker.record_latency(&2u32, Duration::from_millis(500), now);
        }

        let t = timeout.select_timeout(&mut tracker, &[1u32, 2u32], 1, now);
        assert!(
            t >= Duration::from_millis(990) && t <= Duration::from_millis(1010),
            "timeout was {t:?}"
        );
    }

    #[test]
    fn attempt_multiplier_increases_timeout() {
        let now = Instant::now();
        let (mut tracker, timeout) = make_tracker_and_timeout();

        for _ in 0..100 {
            tracker.record_latency(&1u32, Duration::from_millis(50), now);
        }

        let t1 = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
        let t2 = timeout.select_timeout(&mut tracker, &[1u32], 2, now);
        let t3 = timeout.select_timeout(&mut tracker, &[1u32], 3, now);

        assert_eq!(t1, Duration::from_millis(100));
        assert_eq!(t2, Duration::from_millis(200));
        assert_eq!(t3, Duration::from_millis(400));
    }

    #[test]
    fn mixed_data_and_no_data_destinations() {
        let now = Instant::now();
        let (mut tracker, timeout) = make_tracker_and_timeout();

        for _ in 0..100 {
            tracker.record_latency(&1u32, Duration::from_millis(50), now);
        }

        let t = timeout.select_timeout(&mut tracker, &[1u32, 2u32], 1, now);
        assert_eq!(t, Duration::from_millis(100));
    }

    #[test]
    fn ms_variants_match_duration_variants() {
        let now = Instant::now();
        let (mut tracker, timeout) = make_tracker_and_timeout();

        for _ in 0..100 {
            tracker.record_latency(&1u32, Duration::from_millis(50), now);
        }

        let dur = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
        let ms = timeout.select_timeout_ms(&mut tracker, &[1u32], 1, now);
        assert_eq!(dur, Duration::from_millis(ms));

        let dur_fb = timeout.exponential_backoff(3);
        let ms_fb = timeout.exponential_backoff_ms(3);
        assert_eq!(dur_fb, Duration::from_millis(ms_fb));
    }

    // -----------------------------------------------------------------------
    // SyncLatencyTracker tests (feature = "sync")
    // -----------------------------------------------------------------------

    #[cfg(feature = "sync")]
    mod sync_tests {
        use std::time::{Duration, Instant};

        use crate::config::{TimeoutConfig, TrackerConfig};
        use crate::parse::BackoffInterval;
        use crate::sync_tracker::SyncLatencyTracker;
        use crate::timeout::AdaptiveTimeout;

        fn make_sync_tracker_and_timeout() -> (SyncLatencyTracker<u32>, AdaptiveTimeout) {
            let tracker_config = TrackerConfig {
                min_samples: 5,
                ..TrackerConfig::default()
            };
            let timeout_config = TimeoutConfig {
                backoff: "10ms..60s".parse::<BackoffInterval>().unwrap(),
                quantile: 0.99,
                safety_factor: 2.0,
            };
            (
                SyncLatencyTracker::new(tracker_config),
                AdaptiveTimeout::new(timeout_config),
            )
        }

        #[test]
        fn sync_fallback_exponential_backoff_no_data() {
            let now = Instant::now();
            let (tracker, timeout) = make_sync_tracker_and_timeout();

            let t1 = timeout.select_timeout_sync(&tracker, &[1u32], 1, now);
            assert_eq!(t1, Duration::from_millis(10));

            let t2 = timeout.select_timeout_sync(&tracker, &[1u32], 2, now);
            assert_eq!(t2, Duration::from_millis(20));

            let t3 = timeout.select_timeout_sync(&tracker, &[1u32], 3, now);
            assert_eq!(t3, Duration::from_millis(40));
        }

        #[test]
        fn sync_adaptive_timeout_with_data() {
            let now = Instant::now();
            let (tracker, timeout) = make_sync_tracker_and_timeout();

            for _ in 0..100 {
                tracker.record_latency(&1u32, Duration::from_millis(50), now);
            }

            // p99 ~50ms, safety_factor=2, attempt=1: 2 * 50 * 1 = 100ms
            let t = timeout.select_timeout_sync(&tracker, &[1u32], 1, now);
            assert_eq!(t, Duration::from_millis(100));
        }

        #[test]
        fn sync_respects_floor_and_ceiling() {
            let now = Instant::now();
            let (tracker, timeout) = make_sync_tracker_and_timeout();

            // Floor: tiny latency clamped to min_timeout
            for _ in 0..100 {
                tracker.record_latency(&1u32, Duration::from_millis(1), now);
            }
            let t = timeout.select_timeout_sync(&tracker, &[1u32], 1, now);
            assert_eq!(t, Duration::from_millis(10));

            // Ceiling: huge latency clamped to max_timeout
            for _ in 0..100 {
                tracker.record_latency(&2u32, Duration::from_millis(50_000), now);
            }
            let t = timeout.select_timeout_sync(&tracker, &[2u32], 1, now);
            assert_eq!(t, Duration::from_secs(60));
        }

        #[test]
        fn sync_max_across_destinations() {
            let now = Instant::now();
            let (tracker, timeout) = make_sync_tracker_and_timeout();

            for _ in 0..100 {
                tracker.record_latency(&1u32, Duration::from_millis(10), now);
                tracker.record_latency(&2u32, Duration::from_millis(500), now);
            }

            let t = timeout.select_timeout_sync(&tracker, &[1u32, 2u32], 1, now);
            assert!(
                t >= Duration::from_millis(990) && t <= Duration::from_millis(1010),
                "timeout was {t:?}"
            );
        }

        #[test]
        fn sync_ms_variants_match_duration_variants() {
            let now = Instant::now();
            let (tracker, timeout) = make_sync_tracker_and_timeout();

            for _ in 0..100 {
                tracker.record_latency(&1u32, Duration::from_millis(50), now);
            }

            let dur = timeout.select_timeout_sync(&tracker, &[1u32], 1, now);
            let ms = timeout.select_timeout_sync_ms(&tracker, &[1u32], 1, now);
            assert_eq!(dur, Duration::from_millis(ms));
        }

        #[test]
        fn sync_matches_mutable_tracker_results() {
            use crate::tracker::LatencyTracker;

            let now = Instant::now();
            let tracker_config = TrackerConfig {
                min_samples: 5,
                ..TrackerConfig::default()
            };
            let timeout_config = TimeoutConfig {
                backoff: "10ms..60s".parse::<BackoffInterval>().unwrap(),
                quantile: 0.99,
                safety_factor: 2.0,
            };

            let mut mutable_tracker = LatencyTracker::<u32, Instant>::new(tracker_config);
            let sync_tracker = SyncLatencyTracker::<u32>::new(tracker_config);
            let timeout = AdaptiveTimeout::new(timeout_config);

            // Same data in both trackers.
            for _ in 0..100 {
                mutable_tracker.record_latency(&1u32, Duration::from_millis(50), now);
                sync_tracker.record_latency(&1u32, Duration::from_millis(50), now);
            }

            let ms_mut = timeout.select_timeout_ms(&mut mutable_tracker, &[1u32], 1, now);
            let ms_sync = timeout.select_timeout_sync_ms(&sync_tracker, &[1u32], 1, now);
            assert_eq!(ms_mut, ms_sync);
        }
    }
}
