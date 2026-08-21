use std::fmt::{Display, Error, Formatter};

use crate::{Authority, Host, IPAddress};

/// Represents an IP address with an associated port.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct SocketAddress {
    /// The IP address.
    ip: IPAddress,

    /// The port.
    port: u16,
}

impl SocketAddress {
    //! Constructors

    /// Creates a new socket address.
    pub const fn new(ip: IPAddress, port: u16) -> Self {
        Self { ip, port }
    }
}

impl SocketAddress {
    //! Conversions

    /// Converts the address to an authority.
    pub const fn to_authority(&self) -> Authority {
        Authority::Address(*self)
    }
}

impl<A: Into<IPAddress>> From<(A, u16)> for SocketAddress {
    fn from(tuple: (A, u16)) -> Self {
        Self {
            ip: tuple.0.into(),
            port: tuple.1,
        }
    }
}

impl SocketAddress {
    //! Properties

    /// Gets the IP address.
    pub const fn ip(&self) -> IPAddress {
        self.ip
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Gets the host.
    pub const fn host(&self) -> Host {
        self.ip.to_host()
    }
}

impl Display for SocketAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self.ip {
            IPAddress::V4(v4) => write!(f, "{}:{}", v4, self.port),
            IPAddress::V6(v6) => write!(f, "[{}]:{}", v6, self.port),
        }
    }
}

#[cfg(test)]
mod constructor_tests {
    use crate::{IPv4Address, SocketAddress};

    #[test]
    fn new() {
        let socket: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        assert_eq!(socket.ip, IPv4Address::LOCALHOST.to_ip());
        assert_eq!(socket.port, 80);
    }
}

#[cfg(test)]
mod conversion_tests {
    use crate::{Authority, IPv4Address};

    #[test]
    fn to_authority() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_socket(80).to_authority(),
            Authority::Address(IPv4Address::LOCALHOST.to_socket(80))
        );
    }
}

#[cfg(test)]
mod from_test {
    use crate::{IPv4Address, SocketAddress};

    #[test]
    fn from_tuple() {
        let socket: SocketAddress = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(
            socket,
            SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80)
        );
    }
}

#[cfg(test)]
mod property_tests {
    use crate::{Host, IPAddress, IPv4Address, SocketAddress};

    #[test]
    fn ip() {
        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();
        assert_eq!(SocketAddress::new(ip, 80).ip(), ip);
    }

    #[test]
    fn port() {
        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();
        assert_eq!(SocketAddress::new(ip, 80).port(), 80);
    }

    #[test]
    fn host() {
        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();
        assert_eq!(SocketAddress::new(ip, 80).host(), Host::Address(ip));
    }
}

#[cfg(test)]
mod display_tests {
    use crate::{IPv4Address, IPv6Address, SocketAddress};

    #[test]
    fn display_v4() {
        assert_eq!(
            SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80).to_string(),
            "127.0.0.1:80"
        );
    }

    #[test]
    fn display_v6() {
        assert_eq!(
            SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80).to_string(),
            "[::1]:80"
        );
    }
}
