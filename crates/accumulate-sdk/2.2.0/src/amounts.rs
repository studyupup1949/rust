//! ACME amount helpers.
//!
//! Accumulate denominates ACME in *base units* where **1 ACME = 1e8 base
//! units**. Passing whole ACME where base units are expected is the single most
//! common integration bug. Use [`Amount`] to convert explicitly.

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
