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
    //! Properties

    /// Checks if the host is a domain.
    pub fn is_domain(&self) -> bool {
        matches!(self, Self::Name(_))
    }

    /// Checks if the host is an IP address.
    pub fn is_ip(&self) -> bool {
        matches!(self, Self::Address(_))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, Host, IPv4Address};

    #[test]
    fn properties() {
        let host: Host = Domain::localhost().into();
        assert!(host.is_domain());
        assert!(!host.is_ip());

        let host: Host = IPv4Address::LOCALHOST.into();
        assert!(!host.is_domain());
        assert!(host.is_ip());
    }
}
