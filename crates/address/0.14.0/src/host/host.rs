use crate::{Domain, IPAddress};

/// Either a domain or an IP address.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Host {
    /// A domain.
    Name(Domain),

    /// An IP address.
    Address(IPAddress),
}

impl From<Domain> for Host {
    fn from(domain: Domain) -> Self {
        Self::Name(domain)
    }
}

impl<A: Into<IPAddress>> From<A> for Host {
    fn from(ip: A) -> Self {
        Self::Address(ip.into())
    }
}

impl Host {
    //! Matching

    /// Checks if the host is a domain.
    pub const fn is_domain(&self) -> bool {
        matches!(self, Self::Name(_))
    }

    /// Checks if the host is an IP address.
    pub const fn is_ip(&self) -> bool {
        matches!(self, Self::Address(_))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, Host, IPAddress, IPv4Address};

    #[test]
    fn construction() {
        let result: Host = Domain::localhost().into();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);

        let result: Host = IPv4Address::LOCALHOST.into();
        let expected: Host = Host::Address(IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(result, expected);
    }

    #[test]
    fn matching() {
        let host: Host = Domain::localhost().into();
        assert!(host.is_domain());
        assert!(!host.is_ip());

        let host: Host = IPv4Address::LOCALHOST.into();
        assert!(!host.is_domain());
        assert!(host.is_ip());
    }
}
