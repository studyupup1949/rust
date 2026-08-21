use crate::{DomainRef, IPAddress};

/// Either a domain name reference or an IP address.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum HostRef<'a> {
    /// A domain name reference.
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
    //! Properties

    /// Checks if the host reference is a domain name reference.
    pub fn is_domain(&self) -> bool {
        matches!(self, Self::Name(_))
    }

    /// Checks if the host reference is an IP address.
    pub fn is_ip(&self) -> bool {
        matches!(self, Self::Address(_))
    }
}

#[cfg(test)]
mod tests {
    use crate::{DomainRef, HostRef, IPv4Address};

    #[test]
    fn properties() {
        let host: HostRef = DomainRef::LOCALHOST.into();
        assert!(host.is_domain());
        assert!(!host.is_ip());

        let host: HostRef = IPv4Address::LOCALHOST.into();
        assert!(!host.is_domain());
        assert!(host.is_ip());
    }
}
