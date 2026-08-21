/// Strategy to handle values received before the interval has elapsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitStrategy {
    /// Drop values that arrive too early.
    Drop,
    /// Wait for the remaining interval before forwarding the value.
    Wait,
}
