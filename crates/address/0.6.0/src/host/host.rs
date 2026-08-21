use crate::{Domain, IPAddress};

/// Represents either a domain or an IP address.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Host {
    /// A Domain
    Name(Domain),

    /// An IP Address
    Address(IPAddress),
}

impl Host {
    //! Matching

    /// Checks if the host is a domain.
    pub fn is_name(&self) -> bool {
        matches!(self, Self::Name(_))
    }

    /// Checks if the host is an IP address.
    pub fn is_address(&self) -> bool {
        matches!(self, Self::Address(_))
    }
}
