/// Specifies how the clock adjusts to the requested target time.
pub trait AdjustmentSettings {
    /// At what rate the current time adjusts to the target time.
    /// The unit can be thought of as "seconds per second",
    /// so if `rate` returns 0.001, the clock will be adjusted by
    /// at most one millisecond per second.
    ///
    /// If `rate` returns 1.0, and the clock has ran ahead,
    /// the elapsed time will stop while waiting for the target time to catch up.
    /// If the rate is grater than 1.0, the elapsed time would go backwards in this case.
    fn rate(&self) -> f64;
}

/// Adjusts the clock at a constant rate.
/// The rate is a ratio, i.e. how many seconds the clock adjusts per second
/// (or other unit of time, as the rate is unitless).
#[derive(Clone)]
pub struct ConstantRate(pub f64);

impl AdjustmentSettings for ConstantRate {
    fn rate(&self) -> f64 {
        self.0
    }
}
