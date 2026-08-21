use crate::{DomainRef, IPAddress};

/// Either a domain reference or an IP address.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum HostRef<'a> {
    /// A domain reference.
    Name(DomainRef<'a>),

    /// An IP address.
    Address(IPAddress),
}

impl<'a> From<DomainRef<'a>> for HostRef<'a> {
    fn from(domain: DomainRef<'a>) -> Self {
        Self::Name(domain)
    }
}

impl<'a, A: Into<IPAddress>> From<A> for HostRef<'a> {
    fn from(ip: A) -> Self {
        Self::Address(ip.into())
    }
}

impl<'a> HostRef<'a> {
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
    use crate::{DomainRef, Host, HostRef, IPAddress, IPv4Address};

    #[test]
    fn construction() {
        let result: HostRef = DomainRef::LOCALHOST.into();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);

        let host: Host = Host::Address(IPAddress::V4(IPv4Address::LOCALHOST));
        let result: HostRef = host.to_ref();
        let expected: HostRef = HostRef::Address(IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(result, expected);
    }

    #[test]
    fn matching() {
        let host: HostRef = DomainRef::LOCALHOST.into();
        assert!(host.is_domain());
        assert!(!host.is_ip());

        let host: HostRef = IPv4Address::LOCALHOST.into();
        assert!(!host.is_domain());
        assert!(host.is_ip());
    }
}
