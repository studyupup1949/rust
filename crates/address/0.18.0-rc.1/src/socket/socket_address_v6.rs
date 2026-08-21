use crate::IPv6Address;

/// An IPv6 address with an associated port.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
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

impl SocketAddressV6 {
    //! Properties

    /// Gets the IPv6 address.
    pub const fn ip(&self) -> IPv6Address {
        self.ip
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use crate::socket::SocketAddressV6;
    use crate::IPv6Address;

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
    fn properties() {
        let socket: SocketAddressV6 = (IPv6Address::LOCALHOST, 80).into();
        assert_eq!(socket.ip(), IPv6Address::LOCALHOST);
        assert_eq!(socket.port(), 80);
    }
}
