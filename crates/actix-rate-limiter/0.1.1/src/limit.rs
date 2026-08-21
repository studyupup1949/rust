//! Representation and implementation of the bucket and its limitx.

/// Builder for the `Limit`.
#[derive(Copy, Clone, Debug)]
pub struct LimitBuilder {
    ttl: i64,
    amount: i64,
}

impl LimitBuilder {
    /// Creates a new instance of `LimitBuilder`.
    pub fn new() -> Self {
        Self { ttl: 1, amount: 5 }
    }

    /// Sets the TTL (in seconds) for the buckets.
    pub fn set_ttl(&mut self, ttl: i64) -> &mut Self {
        self.ttl = ttl;
        self
    }

    /// Sets the amount of requests that one specific bucket can accept in the TTL.
    pub fn set_amount(&mut self, amount: i64) -> &mut Self {
        self.amount = amount;
        self
    }

    /// Builds the instance of the builder and converts it into a `Route`
    /// instance with the set values.
    pub fn build(self) -> Limit {
        Limit {
            ttl: self.ttl,
            amount: self.amount,
        }
    }
}

impl Default for LimitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// General type for limits inside the rate limiter. The default rate is 5 RPS.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Limit {
    /// TTL in seconds for each individual bucket.
    pub ttl: i64,
    /// The amount of requests that one specific bucket can accept in the TTL.
    pub amount: i64,
}

impl Default for Limit {
    fn default() -> Self {
        Self { ttl: 1, amount: 5 }
    }
}
