use crate::IPv6Address;

/// An IPv6 address with an associated port.
#[must_use]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SocketAddressV6 {
    ip: IPv6Address,
    port: u16,
}

impl SocketAddressV6 {
    //! Construction

    /// Creates a new IPv6 socket address.
    pub const fn new(ip: IPv6Address, port: u16) -> Self {
        Self { ip, port }
    }
}

impl<A: Into<IPv6Address>> From<(A, u16)> for SocketAddressV6 {
    fn from(tuple: (A, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1)
    }
}

impl From<SocketAddressV6> for (IPv6Address, u16) {
    fn from(socket: SocketAddressV6) -> Self {
        (socket.ip, socket.port)
    }
}

impl SocketAddressV6 {
    //! Properties

    /// Gets the IPv6 address.
    pub const fn ip(self) -> IPv6Address {
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
    use crate::{IPv6Address, SocketAddressV6};

    #[test]
    fn construction() {
        let socket: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
        assert_eq!(socket.ip, IPv6Address::LOCALHOST);
        assert_eq!(socket.port, 80);

        let socket: SocketAddressV6 = (IPv6Address::LOCALHOST, 80).into();
        assert_eq!(socket.ip, IPv6Address::LOCALHOST);
        assert_eq!(socket.port, 80);
    }

    #[test]
    fn deconstruction() {
        let socket: SocketAddressV6 = (IPv6Address::LOCALHOST, 80).into();
        let (ip, port): (IPv6Address, u16) = socket.into();
        assert_eq!(ip, IPv6Address::LOCALHOST);
        assert_eq!(port, 80);
    }

    #[test]
    fn properties() {
        let socket: SocketAddressV6 = (IPv6Address::LOCALHOST, 80).into();
        assert_eq!(socket.ip(), IPv6Address::LOCALHOST);
        assert_eq!(socket.port(), 80);
    }
}
