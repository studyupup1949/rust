use crate::{IPv4Address, IPv6Address};

/// Represents an either an IPv4 address or an IPv6 address.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum IPAddress {
    /// An IPv4 Address
    V4(IPv4Address),

    /// An IPv6 Address
    V6(IPv6Address),
}

impl IPAddress {
    //! Matching

    /// Checks if the address is an IPv4 address.
    pub const fn is_v4(&self) -> bool {
        matches!(self, Self::V4(_))
    }

    /// Checks if the address is an IPv6 address.
    pub const fn is_v6(&self) -> bool {
        matches!(self, Self::V6(_))
    }
}
