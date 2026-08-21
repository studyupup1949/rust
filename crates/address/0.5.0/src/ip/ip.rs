use std::fmt::{Display, Error, Formatter};
use std::net::IpAddr;

use crate::{Authority, Host, IPv4Address, IPv6Address, SocketAddress};

/// Represents either an IPv4 address or an IPv6 address.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum IPAddress {
    /// An IPv4 Address
    V4(IPv4Address),

    /// An IPv6 Address
    V6(IPv6Address),
}

impl IPAddress {
    //! Conversions

    /// Converts the address to an optional IPv4 address.
    pub fn to_v4(&self) -> Option<IPv4Address> {
        match self {
            IPAddress::V4(v4) => Some(*v4),
            _ => None,
        }
    }

    /// Converts the address to an optional IPv6 address.
    pub fn to_v6(&self) -> Option<IPv6Address> {
        match self {
            IPAddress::V6(v6) => Some(*v6),
            _ => None,
        }
    }

    /// Converts the address to a socket address.
    pub const fn to_socket(&self, port: u16) -> SocketAddress {
        SocketAddress::new(*self, port)
    }

    /// Converts the address to a host.
    pub const fn to_host(&self) -> Host {
        Host::Address(*self)
    }

    /// Converts the address to an authority with the port.
    pub const fn to_authority(&self, port: u16) -> Authority {
        Authority::Address(self.to_socket(port))
    }
}

impl From<IPv4Address> for IPAddress {
    fn from(v4: IPv4Address) -> Self {
        IPAddress::V4(v4)
    }
}

impl From<IPv6Address> for IPAddress {
    fn from(v6: IPv6Address) -> Self {
        IPAddress::V6(v6)
    }
}

impl IPAddress {
    //! Standard Library Conversions

    /// Converts the address to a standard library address.
    pub fn to_std(&self) -> IpAddr {
        match self {
            IPAddress::V4(v4) => IpAddr::V4(v4.to_std()),
            IPAddress::V6(v6) => IpAddr::V6(v6.to_std()),
        }
    }
}

impl From<IpAddr> for IPAddress {
    fn from(std: IpAddr) -> Self {
        match std {
            IpAddr::V4(v4) => IPv4Address::from(v4).into(),
            IpAddr::V6(v6) => IPv6Address::from(v6).into(),
        }
    }
}

impl From<IPAddress> for IpAddr {
    fn from(ip: IPAddress) -> Self {
        match ip {
            IPAddress::V4(v4) => IpAddr::V4(v4.to_std()),
            IPAddress::V6(v6) => IpAddr::V6(v6.to_std()),
        }
    }
}

impl IPAddress {
    //! Matching

    /// Checks if the address is an IPv4 address.
    pub fn is_v4(&self) -> bool {
        match self {
            IPAddress::V4(_) => true,
            _ => false,
        }
    }

    /// Checks if the address is an IPv6 address.
    pub fn is_v6(&self) -> bool {
        match self {
            IPAddress::V6(_) => true,
            _ => false,
        }
    }
}

impl IPAddress {
    //! Classifications

    /// Checks if the address is an unspecified address. (::) or (0.0.0.0)
    pub fn is_unspecified(&self) -> bool {
        match self {
            IPAddress::V4(v4) => v4.is_unspecified(),
            IPAddress::V6(v6) => v6.is_unspecified(),
        }
    }

    /// Checks if the address is a loopback address. (::1) or (127.0.0.0/8)
    pub fn is_loopback(&self) -> bool {
        match self {
            IPAddress::V4(v4) => v4.is_loopback(),
            IPAddress::V6(v6) => v6.is_loopback(),
        }
    }
}

impl Display for IPAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            IPAddress::V4(v4) => v4.fmt(f),
            IPAddress::V6(v6) => v6.fmt(f),
        }
    }
}

#[cfg(test)]
mod conversion_tests {
    use crate::{Host, IPv4Address, IPv6Address, SocketAddress};

    #[test]
    fn to_v4() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_ip().to_v4(),
            Some(IPv4Address::LOCALHOST)
        );
        assert_eq!(IPv6Address::LOCALHOST.to_ip().to_v4(), None);
    }

    #[test]
    fn to_v6() {
        assert_eq!(IPv4Address::LOCALHOST.to_ip().to_v6(), None);
        assert_eq!(
            IPv6Address::LOCALHOST.to_ip().to_v6(),
            Some(IPv6Address::LOCALHOST)
        );
    }

    #[test]
    fn to_socket() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_ip().to_socket(80),
            SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80)
        );
    }

    #[test]
    fn to_host() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_ip().to_host(),
            Host::Address(IPv4Address::LOCALHOST.to_ip())
        );
    }
}

#[cfg(test)]
mod from_tests {
    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn from_v4() {
        assert_eq!(
            IPAddress::from(IPv4Address::LOCALHOST),
            IPAddress::V4(IPv4Address::LOCALHOST)
        );
    }

    #[test]
    fn from_v6() {
        assert_eq!(
            IPAddress::from(IPv6Address::LOCALHOST),
            IPAddress::V6(IPv6Address::LOCALHOST)
        );
    }
}

#[cfg(test)]
mod std_tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn to_std() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_ip().to_std(),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        );
        assert_eq!(
            IPv6Address::LOCALHOST.to_ip().to_std(),
            IpAddr::V6(Ipv6Addr::LOCALHOST)
        );
    }

    #[test]
    fn from_std() {
        assert_eq!(
            IPAddress::from(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            IPv4Address::LOCALHOST.to_ip()
        );
        assert_eq!(
            IPAddress::from(IpAddr::V6(Ipv6Addr::LOCALHOST)),
            IPv6Address::LOCALHOST.to_ip()
        );
    }

    #[test]
    fn std_from_ip() {
        assert_eq!(
            IpAddr::from(IPv4Address::LOCALHOST.to_ip()),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        );
        assert_eq!(
            IpAddr::from(IPv6Address::LOCALHOST.to_ip()),
            IpAddr::V6(Ipv6Addr::LOCALHOST)
        );
    }
}

#[cfg(test)]
mod matching_tests {
    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn is_v4() {
        assert_eq!(IPAddress::V4(IPv4Address::LOCALHOST).is_v4(), true);
        assert_eq!(IPAddress::V6(IPv6Address::LOCALHOST).is_v4(), false);
    }

    #[test]
    fn is_v6() {
        assert_eq!(IPAddress::V4(IPv4Address::LOCALHOST).is_v6(), false);
        assert_eq!(IPAddress::V6(IPv6Address::LOCALHOST).is_v6(), true);
    }
}

#[cfg(test)]
mod classification_tests {
    use crate::{IPv4Address, IPv6Address};

    #[test]
    fn is_unspecified() {
        assert_eq!(IPv4Address::UNSPECIFIED.to_ip().is_unspecified(), true);
        assert_eq!(IPv4Address::LOCALHOST.to_ip().is_unspecified(), false);
        assert_eq!(IPv6Address::UNSPECIFIED.to_ip().is_unspecified(), true);
        assert_eq!(IPv6Address::LOCALHOST.to_ip().is_unspecified(), false);
    }

    #[test]
    fn is_loopback() {
        assert_eq!(IPv4Address::UNSPECIFIED.is_loopback(), false);
        assert_eq!(IPv4Address::LOCALHOST.is_loopback(), true);
        assert_eq!(IPv6Address::UNSPECIFIED.is_loopback(), false);
        assert_eq!(IPv6Address::LOCALHOST.is_loopback(), true);
    }
}

#[cfg(test)]
mod display_tests {
    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn display_v4() {
        assert_eq!(
            IPAddress::from(IPv4Address::LOCALHOST).to_string(),
            "127.0.0.1"
        );
    }

    #[test]
    fn display_v6() {
        assert_eq!(IPAddress::from(IPv6Address::LOCALHOST).to_string(), "::1");
    }
}
