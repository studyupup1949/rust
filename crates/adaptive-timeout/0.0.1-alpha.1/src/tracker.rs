use std::borrow::Borrow;
use std::collections::HashMap as StdHashMap;
use std::hash::Hash;
use std::time::Duration;

/// Non-cryptographic hasher for internal maps. Keys are application-supplied
/// identifiers, not untrusted input.
type HashMap<K, V> = StdHashMap<K, V, foldhash::fast::RandomState>;

use crate::clock;
use crate::config::{SIGNIFICANT_VALUE_DIGITS, TrackerConfig};
use crate::histogram::SlidingWindowHistogram;

/// Default number of sub-windows for the sliding window histogram.
///
/// With the default `window_ms` of 60 seconds, this gives 6-second
/// sub-windows. See [`LatencyTracker`] for details on the tradeoffs of
/// changing this value.
pub const DEFAULT_SUB_WINDOWS: usize = 10;

/// Tracks latencies per destination and provides quantile estimates.
///
/// Each destination gets its own sliding-window histogram. Create one tracker
/// per service/operation type to track latencies independently.
///
/// # Type parameters
///
/// - `D` — destination key (node, endpoint, shard, …).
/// - `I` — time source, defaults to [`std::time::Instant`].
/// - `N` — number of sub-windows in each sliding-window histogram, defaults to
///   [`DEFAULT_SUB_WINDOWS`] (10).
///
/// # Sliding window and sub-windows
///
/// Think of it like the Linux load average displayed by `top`: the 1-min,
/// 5-min, and 15-min averages all track the same metric but react to changes
/// at different speeds. A shorter window (1-min) catches spikes quickly but
/// is noisy; a longer window (15-min) is smoother but slow to reflect new
/// conditions.
///
/// Here, two settings control this behaviour:
///
/// [`window_ms`](crate::TrackerConfig::window_ms) sets **how far back the
/// histogram looks** (default: 60s). This is the "memory" of the tracker —
/// like choosing between a 1-min or 15-min load average:
///
/// - **Longer window** (e.g. 5 min): more samples, more stable estimates,
///   but slower to react to sudden latency changes. Good for low-traffic
///   destinations where samples arrive infrequently.
///
/// - **Shorter window** (e.g. 5s): reacts quickly to latency shifts, but
///   with fewer samples the estimates are noisier. May drop below
///   [`min_samples`](crate::TrackerConfig::min_samples) during traffic lulls,
///   causing the system to fall back to exponential backoff.
///
/// `N` controls **how smoothly old data is shed** as the window slides
/// forward. The window is divided into `N` equal sub-windows, and as time
/// advances, sub-windows rotate out one at a time:
///
/// - **Higher `N`** (e.g. 20): samples expire in small increments
///   (`window / N` per step), so quantile estimates transition smoothly —
///   like a load average that updates every few seconds. The cost is more
///   memory (each sub-window is a separate HdrHistogram allocation).
///
/// - **Lower `N`** (e.g. 3): samples expire in large chunks. Uses less
///   memory, but a bigger fraction of data disappears at once — like a load
///   average that only updates every few minutes, causing jumpy readings.
///
/// - **`N = 1`**: the entire window expires at once — a tumbling window. The
///   histogram alternates between "full of data" and "completely empty" each
///   `window_ms` period.
///
/// The two settings interact: the effective rotation interval is
/// `window_ms / N`. With the defaults (`window_ms = 60_000`, `N = 10`), each
/// sub-window covers 6 seconds — old data is shed in 10% increments every
/// 6 seconds. If you want finer granularity (e.g. 1-second rotations at a
/// 60s window), set `N = 60`.
///
/// # Example
///
/// ```
/// use std::time::{Duration, Instant};
/// use adaptive_timeout::LatencyTracker;
///
/// let now = Instant::now();
/// let mut tracker = LatencyTracker::<u32>::with_defaults(now);
///
/// for _ in 0..100 {
///     tracker.record_latency_ms(&1u32, 50, now);
/// }
///
/// let p99 = tracker.quantile_ms(&1u32, 0.99, now);
/// assert_eq!(p99, Some(50));
/// ```
pub struct LatencyTracker<
    D,
    I: clock::Instant = std::time::Instant,
    const N: usize = DEFAULT_SUB_WINDOWS,
> {
    config: TrackerConfig,
    histograms: HashMap<D, SlidingWindowHistogram<I, N>>,
}

impl<D, I, const N: usize> LatencyTracker<D, I, N>
where
    D: Hash + Eq + Clone,
    I: clock::Instant,
{
    /// Creates a new tracker with the given configuration.
    pub fn new(config: TrackerConfig, _now: I) -> Self {
        Self {
            config,
            histograms: HashMap::default(),
        }
    }

    /// Creates a new tracker with default configuration.
    pub fn with_defaults(now: I) -> Self {
        Self::new(TrackerConfig::default(), now)
    }

    /// Records a latency sample given two instants. Returns the computed
    /// duration.
    ///
    /// ```
    /// # use std::time::{Duration, Instant};
    /// # use adaptive_timeout::LatencyTracker;
    /// let now = Instant::now();
    /// let mut tracker = LatencyTracker::<u32>::with_defaults(now);
    /// let later = now + Duration::from_millis(42);
    /// let latency = tracker.record_latency_at(&1u32, now, later);
    /// assert_eq!(latency, Duration::from_millis(42));
    /// ```
    #[inline]
    pub fn record_latency_at<Q>(&mut self, dest: &Q, earlier: I, now: I) -> Duration
    where
        D: Borrow<Q>,
        Q: Hash + Eq + ToOwned<Owned = D> + ?Sized,
    {
        let latency = now.duration_since(earlier);
        self.record_latency_ms(dest, latency.as_millis() as u64, now);
        latency
    }

    /// Records a latency sample as a [`Duration`].
    #[inline]
    pub fn record_latency<Q>(&mut self, dest: &Q, latency: Duration, now: I)
    where
        D: Borrow<Q>,
        Q: Hash + Eq + ToOwned<Owned = D> + ?Sized,
    {
        self.record_latency_ms(dest, latency.as_millis() as u64, now);
    }

    /// Records a latency sample in milliseconds. This is the fastest
    /// recording path — no `Duration` conversion, no allocation on the
    /// hot path (destination already seen).
    #[inline]
    pub fn record_latency_ms<Q>(&mut self, dest: &Q, latency_ms: u64, now: I)
    where
        D: Borrow<Q>,
        Q: Hash + Eq + ToOwned<Owned = D> + ?Sized,
    {
        if let Some(histogram) = self.histograms.get_mut(dest) {
            histogram.record(latency_ms, now);
            return;
        }
        self.record_latency_ms_cold(dest.to_owned(), latency_ms, now);
    }

    #[cold]
    fn record_latency_ms_cold(&mut self, dest: D, latency_ms: u64, now: I) {
        let mut histogram = self.new_histogram(now);
        histogram.record(latency_ms, now);
        self.histograms.insert(dest, histogram);
    }

    /// Returns the estimated latency in milliseconds at the given quantile,
    /// or `None` if insufficient data.
    #[inline]
    pub fn quantile_ms<Q>(&mut self, dest: &Q, quantile: f64, now: I) -> Option<u64>
    where
        D: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let histogram = self.histograms.get_mut(dest)?;
        histogram.quantile(quantile, self.config.min_samples as u64, now)
    }

    /// Returns the estimated latency as a [`Duration`] at the given quantile,
    /// or `None` if insufficient data.
    #[inline]
    pub fn quantile<Q>(&mut self, dest: &Q, quantile: f64, now: I) -> Option<Duration>
    where
        D: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.quantile_ms(dest, quantile, now)
            .map(Duration::from_millis)
    }

    /// Clears all tracked state.
    pub fn clear(&mut self) {
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

    type TestTracker = LatencyTracker<u32>;

    fn make_tracker(now: Instant) -> TestTracker {
        let config = TrackerConfig {
            min_samples: 5,
            ..TrackerConfig::default()
        };
        LatencyTracker::new(config, now)
    }

    #[test]
    fn no_data_returns_none() {
        let now = Instant::now();
        let mut tracker = make_tracker(now);
        assert_eq!(tracker.quantile(&1, 0.5, now), None);
    }

    #[test]
    fn record_latency_directly() {
        let now = Instant::now();
        let mut tracker = make_tracker(now);

        for _ in 0..10 {
            tracker.record_latency(&1, Duration::from_millis(100), now);
        }

        let p50 = tracker.quantile(&1, 0.5, now).unwrap();
        assert_eq!(p50, Duration::from_millis(100));
    }

    #[test]
    fn record_latency_ms_directly() {
        let now = Instant::now();
        let mut tracker = make_tracker(now);

        for _ in 0..10 {
            tracker.record_latency_ms(&1, 100, now);
        }

        let p50 = tracker.quantile_ms(&1, 0.5, now).unwrap();
        assert_eq!(p50, 100);
    }

    #[test]
    fn record_latency_at_computes_duration() {
        let now = Instant::now();
        let mut tracker = make_tracker(now);
        let later = now + Duration::from_millis(42);

        for _ in 0..10 {
            let d = tracker.record_latency_at(&1, now, later);
            assert_eq!(d, Duration::from_millis(42));
        }

        let p50 = tracker.quantile_ms(&1, 0.5, later).unwrap();
        assert_eq!(p50, 42);
    }

    #[test]
    fn per_destination_isolation() {
        let now = Instant::now();
        let mut tracker = make_tracker(now);

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
        let mut tracker = make_tracker(now);

        for _ in 0..10 {
            tracker.record_latency(&1, Duration::from_millis(100), now);
        }

        tracker.clear();

        assert_eq!(tracker.quantile(&1, 0.5, now), None);
    }

    #[test]
    fn insufficient_samples_returns_none() {
        let now = Instant::now();
        let mut tracker = make_tracker(now); // min_samples = 5

        for _ in 0..4 {
            tracker.record_latency(&1, Duration::from_millis(100), now);
        }

        assert_eq!(tracker.quantile(&1, 0.5, now), None);

        // 5th sample tips it over.
        tracker.record_latency(&1, Duration::from_millis(100), now);
        assert!(tracker.quantile(&1, 0.5, now).is_some());
    }
}
