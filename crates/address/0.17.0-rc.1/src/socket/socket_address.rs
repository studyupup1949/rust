use crate::IPAddress;

/// An IP address with an associated port.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct SocketAddress {
    ip: IPAddress,
    port: u16,
}

impl SocketAddress {
    //! Construction

    /// Creates a new socket address.
    pub const fn new(ip: IPAddress, port: u16) -> Self {
        Self { ip, port }
    }
}

impl<A: Into<IPAddress>> From<(A, u16)> for SocketAddress {
    fn from(tuple: (A, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1)
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
}

#[cfg(test)]
mod tests {
    use crate::socket::SocketAddress;
    use crate::{IPAddress, IPv4Address};

    #[test]
    fn construction() {
        let socket: SocketAddress = SocketAddress::new(IPAddress::V4(IPv4Address::LOCALHOST), 80);
        assert_eq!(socket.ip, IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(socket.port, 80);

        let socket: SocketAddress = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(socket.ip, IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(socket.port, 80);
    }

    #[test]
    fn properties() {
        let socket: SocketAddress = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(socket.ip(), IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(socket.port(), 80);
    }
}
