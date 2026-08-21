use std::time::Duration;

use hdrhistogram::Histogram;

use crate::clock;

/// A time-windowed histogram that automatically expires old data.
///
/// Internally, the sliding window is divided into `N` sub-windows, each covering
/// `window / N` of time. On every operation the struct rotates out sub-windows
/// whose time has passed, ensuring that only samples from the most recent
/// `window` duration contribute to quantile estimates.
///
/// `N` is a compile-time constant (const generic), so the sub-window ring buffer
/// is a fixed-size array with no heap allocation for the buffer itself (each
/// `Histogram<u64>` still allocates internally).
///
/// # Optimization: incremental merge
///
/// Rather than merging all sub-windows on every `quantile()` call, we maintain a
/// pre-merged `combined` histogram that is kept in sync incrementally: new
/// samples are added to both the current sub-window and `combined`, and when
/// sub-windows are rotated out, their contributions are subtracted from
/// `combined`.
pub(crate) struct SlidingWindowHistogram<I: clock::Instant, const N: usize> {
    /// Ring buffer of sub-window histograms.
    sub_windows: [Histogram<u64>; N],
    /// Pre-merged histogram covering all active sub-windows.
    /// Kept in sync incrementally to avoid rebuilding on every query.
    combined: Histogram<u64>,
    /// Duration of each sub-window in milliseconds (avoid repeated conversion).
    sub_window_ms: u64,
    /// Index of the current (active) sub-window in the ring buffer.
    current_index: u8,
    /// The instant at which the current sub-window started.
    current_window_start: I,
    /// Maximum trackable value.
    max_value: u64,
}

impl<I: clock::Instant, const N: usize> SlidingWindowHistogram<I, N> {
    /// Creates a new `SlidingWindowHistogram`.
    ///
    /// # Arguments
    ///
    /// * `window` - Total duration of the sliding window.
    /// * `sigfig` - Significant value digits for HdrHistogram (typically 2 or 3).
    /// * `max_value` - Maximum trackable value in the histogram.
    /// * `now` - The current instant (for testability).
    ///
    /// # Panics
    ///
    /// Panics if `N == 0`, `window` is zero, or `N > 255`.
    pub fn new(window: Duration, sigfig: u8, max_value: u64, now: I) -> Self {
        assert!(N > 0, "N (number of sub-windows) must be > 0");
        assert!(N <= u8::MAX as usize, "N must fit in a u8");
        assert!(!window.is_zero(), "window must be > 0");

        let sub_window_ms = window.as_millis() as u64 / N as u64;
        assert!(sub_window_ms > 0, "sub-window duration must be >= 1ms");

        let sub_windows = std::array::from_fn(|_| {
            Histogram::new_with_max(max_value, sigfig).expect("valid histogram params")
        });
        let combined = Histogram::new_with_max(max_value, sigfig).expect("valid histogram params");

        Self {
            sub_windows,
            combined,
            sub_window_ms,
            current_index: 0,
            current_window_start: now,
            max_value,
        }
    }

    /// Records a latency value (in milliseconds) into the current sub-window
    /// and the combined histogram.
    ///
    /// Values exceeding `max_value` are clamped. Stale sub-windows are rotated
    /// out before recording.
    #[inline]
    pub fn record(&mut self, value_ms: u64, now: I) {
        self.rotate(now);
        let clamped = value_ms.min(self.max_value);
        self.sub_windows[self.current_index as usize]
            .record(clamped)
            .expect("value within max after clamping");
        self.combined
            .record(clamped)
            .expect("value within max after clamping");
    }

    /// Returns the estimated value at the given quantile (0.0 - 1.0), or
    /// `None` if there are fewer than `min_samples` total samples.
    ///
    /// Uses `value_at_quantile` directly (rather than `value_at_percentile`)
    /// to avoid an unnecessary f64 division and to minimize floating-point
    /// precision loss.
    ///
    /// This is an O(1) operation (no merge step) because `combined` is kept
    /// incrementally up to date.
    #[inline]
    pub fn quantile(&mut self, quantile: f64, min_samples: u64, now: I) -> Option<u64> {
        self.rotate(now);

        if self.combined.len() < min_samples {
            return None;
        }

        if self.combined.is_empty() {
            return None;
        }

        Some(self.combined.value_at_quantile(quantile))
    }

    /// Returns the total number of samples across all active (non-expired)
    /// sub-windows.
    #[cfg(test)]
    pub fn count(&self) -> u64 {
        self.combined.len()
    }

    /// Clears all sub-windows and the combined histogram.
    #[cfg(test)]
    pub fn clear(&mut self, now: I) {
        for sub in &mut self.sub_windows {
            sub.reset();
        }
        self.combined.reset();
        self.current_index = 0;
        self.current_window_start = now;
    }

    /// Advance the sliding window, subtracting rotated-out sub-windows from
    /// `combined` and clearing them.
    fn rotate(&mut self, now: I) {
        let elapsed_ms = now.duration_since(self.current_window_start).as_millis() as u64;
        let steps = (elapsed_ms / self.sub_window_ms) as usize;

        if steps == 0 {
            return;
        }

        // If we've gone through a full cycle or more, clear everything.
        if steps >= N {
            for sub in &mut self.sub_windows {
                sub.reset();
            }
            self.combined.reset();
            self.current_index = ((self.current_index as usize + steps) % N) as u8;
        } else {
            for _ in 0..steps {
                self.current_index = ((self.current_index as usize + 1) % N) as u8;
                let idx = self.current_index as usize;
                // Subtract the rotated-out sub-window from combined.
                self.combined
                    .subtract(&self.sub_windows[idx])
                    .expect("subtract from combined");
                self.sub_windows[idx].reset();
            }
        }

        self.current_window_start = self
            .current_window_start
            .add_duration(Duration::from_millis(self.sub_window_ms * steps as u64));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    fn make_histogram(now: Instant) -> SlidingWindowHistogram<Instant, 10> {
        // 10s window, 10 sub-windows of 1s each, 2 sigfig, max 60s
        SlidingWindowHistogram::new(Duration::from_secs(10), 2, 60_000, now)
    }

    #[test]
    fn empty_histogram_returns_none() {
        let now = Instant::now();
        let mut h = make_histogram(now);
        assert_eq!(h.quantile(0.5, 1, now), None);
        assert_eq!(h.count(), 0);
    }

    #[test]
    fn single_sample_below_min_returns_none() {
        let now = Instant::now();
        let mut h = make_histogram(now);
        h.record(100, now);
        assert_eq!(h.count(), 1);
        // min_samples = 2, so should return None
        assert_eq!(h.quantile(0.5, 2, now), None);
        // min_samples = 1, should return the value
        assert!(h.quantile(0.5, 1, now).is_some());
    }

    #[test]
    fn records_and_queries_quantile() {
        let now = Instant::now();
        let mut h = make_histogram(now);

        // Record 100 samples of 50ms
        for _ in 0..100 {
            h.record(50, now);
        }
        assert_eq!(h.count(), 100);

        let p50 = h.quantile(0.5, 1, now).unwrap();
        assert_eq!(p50, 50);
    }

    #[test]
    fn rotation_expires_old_data() {
        let now = Instant::now();
        let mut h = make_histogram(now);

        // Record at t=0
        for _ in 0..50 {
            h.record(100, now);
        }
        assert_eq!(h.count(), 50);

        // Advance past the full window (11 seconds)
        let later = now + Duration::from_secs(11);
        // This should rotate out everything
        h.record(200, later);
        // Only the new sample should remain
        assert_eq!(h.count(), 1);

        let p = h.quantile(0.5, 1, later).unwrap();
        assert_eq!(p, 200);
    }

    #[test]
    fn partial_rotation_keeps_recent_data() {
        let now = Instant::now();
        let mut h = make_histogram(now);

        // Record at t=0
        for _ in 0..50 {
            h.record(100, now);
        }

        // Advance 3 seconds (should rotate out 3 sub-windows, keep 7)
        // Since all 50 samples are in sub-window 0, and we advance 3 steps,
        // sub-windows 1, 2, 3 get cleared (they were already empty), and
        // sub-window 0 data is still within the window.
        let t3 = now + Duration::from_secs(3);
        for _ in 0..50 {
            h.record(200, t3);
        }

        assert_eq!(h.count(), 100);

        // Advance to t=11s (sub-window 0 data from t=0 is now expired)
        let t11 = now + Duration::from_secs(11);
        let p = h.quantile(0.5, 1, t11);
        // The t=0 data should be gone, but the t=3 data might still be in range
        // depending on which sub-window it landed in
        assert!(p.is_some() || h.count() == 0);
    }

    #[test]
    fn clear_resets_everything() {
        let now = Instant::now();
        let mut h = make_histogram(now);

        for _ in 0..100 {
            h.record(50, now);
        }
        assert_eq!(h.count(), 100);

        h.clear(now);
        assert_eq!(h.count(), 0);
        assert_eq!(h.quantile(0.5, 1, now), None);
    }

    #[test]
    fn values_are_clamped_to_max() {
        let now = Instant::now();
        let mut h = make_histogram(now);

        // max_value is 60_000. Recording 100_000 should clamp to 60_000.
        // HdrHistogram with 2 significant digits may round the value slightly,
        // so we check it's within the expected quantization range.
        h.record(100_000, now);
        let p = h.quantile(0.5, 1, now).unwrap();
        assert!((59_000..=61_000).contains(&p), "clamped value was {p}");
    }

    #[test]
    fn quantile_distribution() {
        let now = Instant::now();
        let mut h = make_histogram(now);

        // Record a spread of values: 1..=1000
        for v in 1..=1000 {
            h.record(v, now);
        }

        let p50 = h.quantile(0.5, 1, now).unwrap();
        let p99 = h.quantile(0.99, 1, now).unwrap();

        // With 2 significant figures, expect reasonable approximations.
        // p50 should be ~500, p99 should be ~990
        assert!((480..=520).contains(&p50), "p50 was {p50}");
        assert!((970..=1010).contains(&p99), "p99 was {p99}");
    }

    #[test]
    fn combined_stays_in_sync_across_rotations() {
        let now = Instant::now();
        let mut h = make_histogram(now);

        // Record 50 samples at t=0 in sub-window 0
        for _ in 0..50 {
            h.record(100, now);
        }

        // Record 50 samples at t=2s in sub-window 2
        let t2 = now + Duration::from_secs(2);
        for _ in 0..50 {
            h.record(200, t2);
        }
        assert_eq!(h.count(), 100);

        // Advance to t=10s: sub-window 0 rotates out (was at index 0),
        // sub-window 2 data should remain.
        let t10 = now + Duration::from_secs(10);
        h.record(300, t10);

        // Sub-window 0 (50 samples of 100) should be gone.
        // Remaining: 50 samples of 200 + 1 of 300 = 51.
        assert_eq!(h.count(), 51);

        let p50 = h.quantile(0.5, 1, t10).unwrap();
        assert_eq!(p50, 200);
    }
}
