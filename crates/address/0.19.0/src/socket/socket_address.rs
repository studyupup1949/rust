use crate::IPAddress;

/// An IP address with an associated port.
#[must_use]
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

impl From<SocketAddress> for (IPAddress, u16) {
    fn from(socket: SocketAddress) -> Self {
        (socket.ip, socket.port)
    }
}

impl SocketAddress {
    //! Properties

    /// Gets the IP address.
    pub const fn ip(self) -> IPAddress {
        self.ip
    }

    /// Gets the port.
    #[must_use]
    pub const fn port(self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPAddress, IPv4Address, SocketAddress};

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
    fn deconstruction() {
        let socket: SocketAddress = (IPv4Address::LOCALHOST, 80).into();
        let (ip, port) = socket.into();
        assert_eq!(ip, IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(port, 80);
    }

    #[test]
    fn properties() {
        let socket: SocketAddress = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(socket.ip(), IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(socket.port(), 80);
    }
}
