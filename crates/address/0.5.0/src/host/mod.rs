use std::fmt::{Debug, Display, Error, Formatter};

use crate::{Authority, Domain, Endpoint, IPAddress, SocketAddress};

/// Represents either a domain or an IP address.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Host<'a> {
    /// A Domain Name
    Name(Domain<'a>),

    /// An IP Address
    Address(IPAddress),
}

impl<'a> Host<'a> {
    //! Matching {

    /// Checks if the host is a domain.
    pub fn is_domain(&self) -> bool {
        match self {
            Host::Name(_) => true,
            _ => false,
        }
    }

    /// Checks if the host is an IP address.
    pub fn is_ip(&self) -> bool {
        match self {
            Host::Address(_) => true,
            _ => false,
        }
    }
}

impl<'a> Host<'a> {
    //! Conversions

    /// Converts the host to an optional domain.
    pub fn to_domain(&self) -> Option<Domain> {
        match self {
            Host::Name(domain) => Some(*domain),
            _ => None,
        }
    }

    /// Converts the host to an optional IP address.
    pub fn to_ip(&self) -> Option<IPAddress> {
        match self {
            Host::Address(ip) => Some(*ip),
            _ => None,
        }
    }

    /// Converts the host to an authority with the port.
    pub fn to_authority(&self, port: u16) -> Authority {
        match self {
            Host::Name(domain) => Authority::Name(Endpoint::new(*domain, port)),
            Host::Address(ip) => Authority::Address(SocketAddress::new(*ip, port)),
        }
    }
}

impl<'a> From<IPAddress> for Host<'a> {
    fn from(ip: IPAddress) -> Self {
        Host::Address(ip)
    }
}

impl<'a> From<Domain<'a>> for Host<'a> {
    fn from(domain: Domain<'a>) -> Self {
        Host::Name(domain)
    }
}

impl<'a> Display for Host<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Host::Address(ip) => write!(f, "{}", ip),
            Host::Name(domain) => write!(f, "{}", domain),
        }
    }
}

#[cfg(test)]
mod matching_tests {
    use crate::{Domain, IPv4Address};

    #[test]
    fn is_domain() {
        assert_eq!(Domain::LOCALHOST.to_host().is_domain(), true);
        assert_eq!(IPv4Address::LOCALHOST.to_host().is_domain(), false);
    }

    #[test]
    fn is_ip() {
        assert_eq!(Domain::LOCALHOST.to_host().is_ip(), false);
        assert_eq!(IPv4Address::LOCALHOST.to_host().is_ip(), true);
    }
}

#[cfg(test)]
mod conversion_tests {
    use crate::{Authority, Domain, IPv4Address};

    #[test]
    fn to_domain() {
        assert_eq!(
            Domain::LOCALHOST.to_host().to_domain(),
            Some(Domain::LOCALHOST)
        );
        assert_eq!(IPv4Address::LOCALHOST.to_host().to_domain(), None);
    }

    #[test]
    fn to_ip() {
        assert_eq!(Domain::LOCALHOST.to_host().to_ip(), None);
        assert_eq!(
            IPv4Address::LOCALHOST.to_host().to_ip(),
            Some(IPv4Address::LOCALHOST.to_ip())
        );
    }

    #[test]
    fn to_authority() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_host().to_authority(80),
            Authority::Address((IPv4Address::LOCALHOST, 80).into())
        );
        assert_eq!(
            Domain::LOCALHOST.to_host().to_authority(80),
            Authority::Name((Domain::LOCALHOST, 80).into())
        );
    }
}

#[cfg(test)]
mod display_test {
    use crate::{Domain, IPv4Address};

    #[test]
    fn display() {
        assert_eq!(Domain::LOCALHOST.to_host().to_string(), "localhost");
        assert_eq!(IPv4Address::LOCALHOST.to_host().to_string(), "127.0.0.1");
    }
}
