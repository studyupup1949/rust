use std::fmt::{Display, Error, Formatter};

use crate::authority::Authority;
use crate::ip::IPAddress;

/// Represents an IP address with an associated port.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct SocketAddress {
    ip: IPAddress,
    port: u16,
}

impl SocketAddress {

    /// Creates a new SocketAddress.
    ///
    /// ```
    /// use address::socket::SocketAddress;
    /// use address::ip::IPv4Address;
    ///
    /// let socket: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.ip(), 80);
    /// assert_eq!(socket.ip(), IPv4Address::LOCALHOST.ip());
    /// assert_eq!(socket.port(), 80);
    /// ```
    pub fn new(ip: IPAddress, port: u16) -> SocketAddress {
        SocketAddress{ ip, port }
    }
}

impl SocketAddress {

    /// Gets the IP address.
    pub fn ip(&self) -> IPAddress {
        self.ip
    }

    /// Gets the port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Gets the authority.
    pub fn authority(&self) -> Authority {
        Authority::Address(*self)
    }
}

impl Display for SocketAddress {

    /// ```
    /// use address::socket::SocketAddress;
    /// use address::ip::{IPv4Address, IPv6Address};
    ///
    /// let socket: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.ip(), 80);
    /// assert_eq!(socket.to_string(), "127.0.0.1:80");
    ///
    /// let socket: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.ip(), 443);
    /// assert_eq!(socket.to_string(), "[::1]:443");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self.ip {
            IPAddress::V4(ip) => write!(f, "{}:{}", ip, self.port),
            IPAddress::V6(ip) => write!(f, "[{}]:{}", ip, self.port),
        }
    }
}
