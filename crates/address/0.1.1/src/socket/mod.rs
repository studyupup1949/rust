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
    /// use address::ip::{IPAddress, IPv4Address};
    /// use address::socket::SocketAddress;
    ///
    /// let ip: IPAddress = IPv4Address::LOCALHOST.ip();
    /// let socket: SocketAddress = SocketAddress::new(ip, 80);
    /// assert_eq!(socket.ip(), ip);
    /// assert_eq!(socket.port(), 80);
    /// ```
    pub fn new(ip: IPAddress, port: u16) -> SocketAddress {
        SocketAddress{ ip, port }
    }
}

impl SocketAddress {

    /// Gets the IP address.
    ///
    /// ```
    /// use address::socket::SocketAddress;
    /// use address::ip::{IPAddress, IPv4Address};
    ///
    /// let ip: IPAddress = IPv4Address::LOCALHOST.ip();
    /// assert_eq!(SocketAddress::new(ip, 80).ip(), ip);
    /// ```
    pub fn ip(&self) -> IPAddress {
        self.ip
    }

    /// Gets the port.
    ///
    /// ```
    /// use address::socket::SocketAddress;
    /// use address::ip::{IPAddress, IPv4Address};
    ///
    /// assert_eq!(SocketAddress::new(IPv4Address::LOCALHOST.ip(), 80).port(), 80);
    /// ```
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl ToString for SocketAddress {

    /// ```
    /// use address::ip::{IPAddress, IPv4Address, IPv6Address};
    /// use address::socket::SocketAddress;
    ///
    /// let ip: IPAddress = IPv4Address::LOCALHOST.ip();
    /// assert_eq!(SocketAddress::new(ip, 80).to_string(), "127.0.0.1:80");
    ///
    /// let ip: IPAddress = IPv6Address::LOCALHOST.ip();
    /// assert_eq!(SocketAddress::new(ip, 80).to_string(), "[::1]:80");
    /// ```
    fn to_string(&self) -> String {
        match self.ip {
            IPAddress::V4(ip) => format!("{}:{}", ip.to_string(), self.port),
            IPAddress::V6(ip) => format!("[{}]:{}", ip.to_string(), self.port),
        }
    }
}
