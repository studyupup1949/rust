use crate::IPv4Address;

/// An IPv4 address with an associated port.
#[must_use]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SocketAddressV4 {
    ip: IPv4Address,
    port: u16,
}

impl SocketAddressV4 {
    //! Construction

    /// Creates a new IPv4 socket address.
    pub const fn new(ip: IPv4Address, port: u16) -> Self {
        Self { ip, port }
    }
}

impl<A: Into<IPv4Address>> From<(A, u16)> for SocketAddressV4 {
    fn from(tuple: (A, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1)
    }
}

impl From<SocketAddressV4> for (IPv4Address, u16) {
    fn from(socket: SocketAddressV4) -> Self {
        (socket.ip, socket.port)
    }
}

impl SocketAddressV4 {
    //! Properties

    /// Gets the IPv4 address.
    pub const fn ip(self) -> IPv4Address {
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
    use crate::{IPv4Address, SocketAddressV4};

    #[test]
    fn construction() {
        let socket: SocketAddressV4 = SocketAddressV4::new(IPv4Address::LOCALHOST, 80);
        assert_eq!(socket.ip, IPv4Address::LOCALHOST);
        assert_eq!(socket.port, 80);

        let socket: SocketAddressV4 = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(socket.ip, IPv4Address::LOCALHOST);
        assert_eq!(socket.port, 80);
    }

    #[test]
    fn deconstruction() {
        let socket: SocketAddressV4 = (IPv4Address::LOCALHOST, 80).into();
        let (ip, port): (IPv4Address, u16) = socket.into();
        assert_eq!(ip, IPv4Address::LOCALHOST);
        assert_eq!(port, 80);
    }

    #[test]
    fn properties() {
        let socket: SocketAddressV4 = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(socket.ip(), IPv4Address::LOCALHOST);
        assert_eq!(socket.port(), 80);
    }
}
