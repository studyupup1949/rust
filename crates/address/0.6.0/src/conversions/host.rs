use crate::{Authority, Domain, Host, IPAddress, IPv4Address, IPv6Address};

impl Host {
    //! Conversions

    /// Converts the host to an optional domain.
    pub fn to_domain(self) -> Option<Domain> {
        match self {
            Self::Name(domain) => Some(domain),
            _ => None,
        }
    }

    /// Converts the host to an optional IP address.
    pub fn to_ip(self) -> Option<IPAddress> {
        match self {
            Self::Address(ip) => Some(ip),
            _ => None,
        }
    }

    /// Converts the host to an authority with the port.
    pub fn to_authority(self, port: u16) -> Authority {
        Authority::new(self, port)
    }
}

impl From<Domain> for Host {
    fn from(domain: Domain) -> Self {
        Self::Name(domain)
    }
}

impl From<IPv4Address> for Host {
    fn from(v4: IPv4Address) -> Self {
        Self::Address(v4.to_ip())
    }
}

impl From<IPv6Address> for Host {
    fn from(v6: IPv6Address) -> Self {
        Self::Address(v6.to_ip())
    }
}

impl From<IPAddress> for Host {
    fn from(ip: IPAddress) -> Self {
        Self::Address(ip)
    }
}
