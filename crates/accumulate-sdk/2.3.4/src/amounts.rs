//! Token amount helpers.
//!
//! Accumulate denominates ACME in *base units* where **1 ACME = 1e8 base
//! units**. Passing whole ACME where base units are expected is the single most
//! common integration bug. Use [`Amount`] to convert explicitly.
//!
//! Custom tokens work the same way but with **their own precision**, declared
//! when the token is created. A token with `precision = 2` stores `10000` base
//! units for `100.00` tokens. Use [`Amount::token`] rather than computing the
//! power of ten by hand — see [`Amount::token`] for why this matters.

/// Number of decimal places in ACME (1 ACME = 10^[`ACME_PRECISION`] base units).
pub const ACME_PRECISION: u32 = 8;

/// Base units in one whole ACME (1e8).
pub const ACME_BASE_UNITS: u64 = 100_000_000;

/// An ACME token amount, stored internally as integer base units.
///
/// # Examples
///
/// ```
/// use accumulate_client::Amount;
///
/// assert_eq!(Amount::acme(5).to_wire(), "500000000");
/// assert_eq!(Amount::base_units(250_000_000).to_acme(), 2.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount {
    base_units: u128,
}

impl Amount {
    /// Create from whole ACME. `Amount::acme(1)` == 1e8 base units.
    #[must_use]
    pub fn acme(whole_acme: u64) -> Self {
        Self {
            base_units: u128::from(whole_acme) * u128::from(ACME_BASE_UNITS),
        }
    }

    /// Create from raw base units (the wire representation).
    #[must_use]
    pub fn base_units(units: u128) -> Self {
        Self { base_units: units }
    }

    /// Create from whole units of a **custom token** with the given precision.
    ///
    /// Custom tokens declare their own precision at creation time; the wire
    /// format is always base units. `Amount::token(1000, 8)` is 1000 whole
    /// tokens = `100000000000` base units.
    ///
    /// Without this helper the only options are hand-computing a power of ten
    /// or passing a raw base-unit string, and both are routinely got wrong:
    /// issuing `1000` against a precision-8 token mints `0.00001` tokens, not
    /// 1000. The value silently differs from intent by eight orders of
    /// magnitude and the transaction succeeds either way.
    ///
    /// # Examples
    ///
    /// ```
    /// use accumulate_client::Amount;
    ///
    /// // 1000 whole tokens of a precision-8 token
    /// assert_eq!(Amount::token(1000, 8).to_wire(), "100000000000");
    /// // 100.00 of a precision-2 token
    /// assert_eq!(Amount::token(100, 2).to_wire(), "10000");
    /// // precision 0 means base units *are* whole tokens
    /// assert_eq!(Amount::token(1000, 0).to_wire(), "1000");
    /// ```
    #[must_use]
    pub fn token(whole_tokens: u64, precision: u32) -> Self {
        Self {
            base_units: u128::from(whole_tokens) * 10u128.pow(precision),
        }
    }

    /// The amount expressed in whole units of a token with the given precision.
    ///
    /// # Examples
    ///
    /// ```
    /// use accumulate_client::Amount;
    ///
    /// assert_eq!(Amount::base_units(100_000_000_000).to_token(8), 1000.0);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn to_token(&self, precision: u32) -> f64 {
        self.base_units as f64 / 10u128.pow(precision) as f64
    }

    /// ACME base units needed to buy `credit_count` credits at `oracle_price`
    /// (the integer oracle value from the network oracle query).
    #[must_use]
    pub fn credits(credit_count: u64, oracle_price: u64) -> Self {
        Self {
            base_units: (u128::from(credit_count) * u128::from(ACME_BASE_UNITS) * 100)
                / u128::from(oracle_price),
        }
    }

    /// The amount as an integer number of base units.
    #[must_use]
    pub fn as_base_units(&self) -> u128 {
        self.base_units
    }

    /// Wire representation: base units as a string (what transaction bodies expect).
    #[must_use]
    pub fn to_wire(&self) -> String {
        self.base_units.to_string()
    }

    /// The amount expressed in whole ACME.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn to_acme(&self) -> f64 {
        self.base_units as f64 / ACME_BASE_UNITS as f64
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base_units)
    }
}
