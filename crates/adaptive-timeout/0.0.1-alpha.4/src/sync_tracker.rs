//! Thread-safe latency tracker using [`DashMap`] for concurrent access.
//!
//! [`SyncLatencyTracker`] provides the same per-destination sliding-window
//! histogram tracking as [`LatencyTracker`](crate::LatencyTracker), but
//! allows concurrent recording and querying from multiple threads without
//! an external `Mutex`.
//!
//! Internally, each destination's histogram lives in a [`DashMap`] entry
//! with per-shard locking. Concurrent operations on **different**
//! destinations never contend. Operations on the **same** destination
//! serialize, but the critical section is very short (~40-80ns).

use std::hash::{BuildHasher, Hash};
use std::time::Duration;

use dashmap::DashMap;

use crate::clock;
use crate::config::{SIGNIFICANT_VALUE_DIGITS, TrackerConfig};
use crate::histogram::SlidingWindowHistogram;
use crate::tracker::DEFAULT_SUB_WINDOWS;

/// Thread-safe latency tracker backed by [`DashMap`].
///
/// Drop-in concurrent replacement for [`LatencyTracker`](crate::LatencyTracker).
/// All methods take `&self` instead of `&mut self`, and the type is both
/// `Send` and `Sync`.
///
/// # Type parameters
///
/// Same as [`LatencyTracker`](crate::LatencyTracker):
/// - `D` — destination key.
/// - `I` — time source (defaults to [`std::time::Instant`]).
/// - `N` — number of sub-windows (defaults to [`DEFAULT_SUB_WINDOWS`]).
///
/// # Example
///
/// ```
/// use std::time::Instant;
/// use adaptive_timeout::SyncLatencyTracker;
///
/// let now = Instant::now();
/// let tracker = SyncLatencyTracker::<u32>::default();
///
/// // Can be called from any thread via &tracker.
/// tracker.record_latency_ms(&1u32, 50, now);
/// let p99 = tracker.quantile_ms(&1u32, 0.99, now);
/// ```
pub struct SyncLatencyTracker<
    D,
    I: clock::Instant = std::time::Instant,
    H = foldhash::fast::RandomState,
    const N: usize = DEFAULT_SUB_WINDOWS,
> {
    config: TrackerConfig,
    histograms: DashMap<D, SlidingWindowHistogram<I, N>, H>,
}

impl<D, I> Default for SyncLatencyTracker<D, I, foldhash::fast::RandomState, DEFAULT_SUB_WINDOWS>
where
    D: Hash + Eq + Clone + Send + Sync,
    I: clock::Instant,
{
    fn default() -> Self {
        Self::new(TrackerConfig::default())
    }
}

impl<D, I, H, const N: usize> SyncLatencyTracker<D, I, H, N>
where
    D: Hash + Eq + Clone + Send + Sync,
    I: clock::Instant,
    H: BuildHasher + Default + Clone,
{
    /// Creates a new tracker with the given configuration.
    pub fn new(config: TrackerConfig) -> Self {
        Self {
            config,
            histograms: DashMap::default(),
        }
    }
}

impl<D, I, H, const N: usize> SyncLatencyTracker<D, I, H, N>
where
    D: Hash + Eq + Clone + Send + Sync,
    I: clock::Instant,
    H: BuildHasher + Clone,
{
    pub fn with_hasher_and_config(hasher: H, config: TrackerConfig) -> Self {
        Self {
            config,
            histograms: DashMap::with_hasher(hasher),
        }
    }

    /// Records a latency sample given two instants. Returns the computed
    /// duration.
    #[inline]
    pub fn record_latency_from(&self, dest: &D, earlier: I, now: I) -> Duration {
        let latency = now.duration_since(earlier);
        self.record_latency_ms(dest, latency.as_millis() as u64, now);
        latency
    }

    /// Records a latency sample as a [`Duration`].
    #[inline]
    pub fn record_latency(&self, dest: &D, latency: Duration, now: I) {
        self.record_latency_ms(dest, latency.as_millis() as u64, now);
    }

    /// Records a latency sample in milliseconds.
    ///
    /// Takes `&self` — safe to call from multiple threads concurrently.
    /// Operations on different destinations proceed in parallel; operations
    /// on the same destination serialize briefly.
    #[inline]
    pub fn record_latency_ms(&self, dest: &D, latency_ms: u64, now: I) {
        if let Some(mut entry) = self.histograms.get_mut(dest) {
            entry.value_mut().record(latency_ms, now);
            return;
        }
        self.record_latency_ms_cold(dest.clone(), latency_ms, now);
    }

    #[cold]
    fn record_latency_ms_cold(&self, dest: D, latency_ms: u64, now: I) {
        let mut histogram = self.new_histogram(now);
        histogram.record(latency_ms, now);
        self.histograms.insert(dest, histogram);
    }

    /// Returns the estimated latency in milliseconds at the given quantile,
    /// or `None` if insufficient data.
    ///
    /// Takes `&self` — safe to call from multiple threads concurrently.
    #[inline]
    pub fn quantile_ms(&self, dest: &D, quantile: f64, now: I) -> Option<u64> {
        let mut entry = self.histograms.get_mut(dest)?;
        entry
            .value_mut()
            .quantile(quantile, self.config.min_samples as u64, now)
    }

    /// Returns the estimated latency as a [`Duration`] at the given quantile,
    /// or `None` if insufficient data.
    #[inline]
    pub fn quantile(&self, dest: &D, quantile: f64, now: I) -> Option<Duration> {
        self.quantile_ms(dest, quantile, now)
            .map(Duration::from_millis)
    }

    /// Clears all tracked state.
    pub fn clear(&self) {
        self.histograms.clear();
    }

    /// Returns a reference to the tracker configuration.
    #[inline]
    pub fn config(&self) -> &TrackerConfig {
        &self.config
    }

    fn new_histogram(&self, now: I) -> SlidingWindowHistogram<I, N> {
        SlidingWindowHistogram::new(
            self.config.window(),
            SIGNIFICANT_VALUE_DIGITS,
            self.config.max_trackable_latency_ms as u64,
            now,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    type TestTracker = SyncLatencyTracker<u32>;

    fn make_tracker() -> TestTracker {
        let config = TrackerConfig {
            min_samples: 5,
            ..TrackerConfig::default()
        };
        SyncLatencyTracker::new(config)
    }

    #[test]
    fn no_data_returns_none() {
        let now = Instant::now();
        let tracker = make_tracker();
        assert_eq!(tracker.quantile(&1, 0.5, now), None);
    }

    #[test]
    fn record_latency_directly() {
        let now = Instant::now();
        let tracker = make_tracker();

        for _ in 0..10 {
            tracker.record_latency(&1, Duration::from_millis(100), now);
        }

        let p50 = tracker.quantile(&1, 0.5, now).unwrap();
        assert_eq!(p50, Duration::from_millis(100));
    }

    #[test]
    fn record_latency_ms_directly() {
        let now = Instant::now();
        let tracker = make_tracker();

        for _ in 0..10 {
            tracker.record_latency_ms(&1, 100, now);
        }

        let p50 = tracker.quantile_ms(&1, 0.5, now).unwrap();
        assert_eq!(p50, 100);
    }

    #[test]
    fn record_latency_from_computes_duration() {
        let now = Instant::now();
        let tracker = make_tracker();
        let later = now + Duration::from_millis(42);

        for _ in 0..10 {
            let d = tracker.record_latency_from(&1, now, later);
            assert_eq!(d, Duration::from_millis(42));
        }

        let p50 = tracker.quantile_ms(&1, 0.5, later).unwrap();
        assert_eq!(p50, 42);
    }

    #[test]
    fn per_destination_isolation() {
        let now = Instant::now();
        let tracker = make_tracker();

        for _ in 0..10 {
            tracker.record_latency(&1, Duration::from_millis(100), now);
            tracker.record_latency(&2, Duration::from_millis(500), now);
        }

        let p1 = tracker.quantile(&1, 0.5, now).unwrap();
        let p2 = tracker.quantile(&2, 0.5, now).unwrap();

        assert_eq!(p1, Duration::from_millis(100));
        assert!(
            p2 >= Duration::from_millis(495) && p2 <= Duration::from_millis(505),
            "p2 was {p2:?}"
        );

        assert_eq!(tracker.quantile(&3, 0.5, now), None);
    }

    #[test]
    fn clear_resets_all_state() {
        let now = Instant::now();
        let tracker = make_tracker();

        for _ in 0..10 {
            tracker.record_latency(&1, Duration::from_millis(100), now);
        }

        tracker.clear();

        assert_eq!(tracker.quantile(&1, 0.5, now), None);
    }

    #[test]
    fn insufficient_samples_returns_none() {
        let now = Instant::now();
        let tracker = make_tracker(); // min_samples = 5

        for _ in 0..4 {
            tracker.record_latency(&1, Duration::from_millis(100), now);
        }

        assert_eq!(tracker.quantile(&1, 0.5, now), None);

        // 5th sample tips it over.
        tracker.record_latency(&1, Duration::from_millis(100), now);
        assert!(tracker.quantile(&1, 0.5, now).is_some());
    }

    #[test]
    fn is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SyncLatencyTracker<u32>>();
        assert_send_sync::<SyncLatencyTracker<String>>();
    }

    // -----------------------------------------------------------------------
    // Concurrent stress tests
    // -----------------------------------------------------------------------

    #[test]
    fn concurrent_record_same_destination() {
        use std::sync::Arc;
        use std::thread;

        let now = Instant::now();
        let tracker = Arc::new(make_tracker());
        let num_threads = 8;
        let samples_per_thread = 1_000;

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let tracker = Arc::clone(&tracker);
                thread::spawn(move || {
                    for _ in 0..samples_per_thread {
                        tracker.record_latency_ms(&1, 50, now);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let p50 = tracker.quantile_ms(&1, 0.5, now).unwrap();
        assert_eq!(p50, 50);
    }

    #[test]
    fn concurrent_record_different_destinations() {
        use std::sync::Arc;
        use std::thread;

        let now = Instant::now();
        let tracker = Arc::new(make_tracker());
        let num_threads = 8;
        let samples_per_thread = 1_000;

        let handles: Vec<_> = (0..num_threads)
            .map(|tid| {
                let tracker = Arc::clone(&tracker);
                thread::spawn(move || {
                    let dest = tid as u32;
                    for _ in 0..samples_per_thread {
                        tracker.record_latency_ms(&dest, (tid as u64 + 1) * 10, now);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        // Each destination should have its own latency.
        for tid in 0..num_threads {
            let dest = tid as u32;
            let expected = (tid as u64 + 1) * 10;
            let p50 = tracker.quantile_ms(&dest, 0.5, now).unwrap();
            assert_eq!(p50, expected, "dest {dest}");
        }
    }

    #[test]
    fn concurrent_read_and_write() {
        use std::sync::Arc;
        use std::thread;

        let now = Instant::now();
        let tracker = Arc::new(make_tracker());

        // Pre-fill with enough data for quantile to return Some.
        for _ in 0..100 {
            tracker.record_latency_ms(&1, 50, now);
        }

        let num_writers = 4;
        let num_readers = 4;
        let iterations = 5_000;

        let mut handles = Vec::new();

        // Writers
        for _ in 0..num_writers {
            let tracker = Arc::clone(&tracker);
            handles.push(thread::spawn(move || {
                for _ in 0..iterations {
                    tracker.record_latency_ms(&1, 50, now);
                }
            }));
        }

        // Readers
        for _ in 0..num_readers {
            let tracker = Arc::clone(&tracker);
            handles.push(thread::spawn(move || {
                for _ in 0..iterations {
                    if let Some(p) = tracker.quantile_ms(&1, 0.5, now) {
                        assert_eq!(p, 50, "unexpected quantile: {p}");
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }
}
