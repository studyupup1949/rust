use std::time::Duration;

/// An instant in time, used for measuring elapsed durations.
///
/// This trait abstracts over the time source so that users can provide their
/// own clock implementations (e.g., for simulated time in tests, or for
/// integration with async runtimes that provide their own `Instant` type).
///
/// The crate never calls "now" internally — callers always pass a timestamp
/// into every method. This trait only needs to support computing the elapsed
/// time between two instants and advancing an instant by a duration.
///
/// # Built-in implementation
///
/// [`std::time::Instant`] implements this trait, so the crate works out of
/// the box with the standard library clock. A type alias
/// [`LatencyTracker`](crate::LatencyTracker) defaults the clock parameter to
/// `Instant` for ergonomic use.
///
/// # Example: custom clock
///
/// ```rust
/// use std::time::Duration;
/// use adaptive_timeout::Instant;
///
/// #[derive(Clone, Copy)]
/// struct FakeInstant(u64); // nanoseconds
///
/// impl Instant for FakeInstant {
///     fn duration_since(&self, earlier: Self) -> Duration {
///         Duration::from_nanos(self.0.saturating_sub(earlier.0))
///     }
///     fn add_duration(&self, duration: Duration) -> Self {
///         FakeInstant(self.0 + duration.as_nanos() as u64)
///     }
/// }
/// ```
pub trait Instant: Copy + Clone {
    /// Returns the duration elapsed from `earlier` to `self`.
    ///
    /// Equivalent to `self - earlier` for `std::time::Instant`. If `self` is
    /// before `earlier` (clock skew), the implementation should return
    /// `Duration::ZERO` rather than panicking.
    fn duration_since(&self, earlier: Self) -> Duration;

    /// Returns a new instant that is `duration` later than `self`.
    ///
    /// Equivalent to `self + duration` for `std::time::Instant`.
    fn add_duration(&self, duration: Duration) -> Self;
}

impl Instant for std::time::Instant {
    #[inline]
    fn duration_since(&self, earlier: Self) -> Duration {
        std::time::Instant::duration_since(self, earlier)
    }

    #[inline]
    fn add_duration(&self, duration: Duration) -> Self {
        *self + duration
    }
}

#[cfg(feature = "tokio")]
impl Instant for tokio::time::Instant {
    fn duration_since(&self, earlier: Self) -> Duration {
        tokio::time::Instant::duration_since(self, earlier)
    }

    fn add_duration(&self, duration: Duration) -> Self {
        *self + duration
    }
}
