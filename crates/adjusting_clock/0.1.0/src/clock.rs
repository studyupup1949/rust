use std::time::{Duration, Instant};

use crate::AdjustmentSettings;

/// Measures the current time.
///
/// All methods that need the current time have an [`std::time::Instant`] parameter.
/// It is expected to nondecreasing, or there may be panics.
/// You normally want to use [`Instant::now()`] for this argument.
#[derive(Clone)]
pub struct Clock<A: AdjustmentSettings + Clone> {
    base: Instant,
    offset: Duration,
    /// The requested relative time at time `base`.
    target: Duration,
    adjustment_settings: A,
}

impl<A: AdjustmentSettings + Clone> Clock<A> {
    /// Start measuring the time, starting at `start_time` (typically [`Instant::now()`]).
    pub fn new(start_time: Instant, adjustment_settings: A) -> Self {
        Clock {
            base: start_time,
            offset: Duration::ZERO,
            target: Duration::ZERO,
            adjustment_settings,
        }
    }

    /// Set the current time, which the clock will start adjusting to.
    pub fn set_target(&mut self, now: Instant, target: Duration) {
        self.offset = self.elapsed(now);
        self.base = now;
        self.target = target;
    }

    /// Get the current elapsed time since the clock started, including any adjustments.
    pub fn elapsed(&self, now: Instant) -> Duration {
        let time_since_base = now.duration_since(self.base);
        let unadjusted = time_since_base + self.offset;
        let current_target = time_since_base + self.target;
        let max_delta = Duration::from_secs_f64(
            time_since_base.as_secs_f64() * self.adjustment_settings.rate(),
        );
        if current_target >= unadjusted {
            Duration::min(current_target, unadjusted + max_delta)
        } else {
            Duration::max(current_target, unadjusted - max_delta)
        }
    }
}
