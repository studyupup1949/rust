use crate::{Authority, SocketAddress, SocketAddressV4, SocketAddressV6};

impl SocketAddressV4 {
    /// Converts the address to a socket address.
    pub const fn to_socket(&self) -> SocketAddress {
        SocketAddress::new(self.ip().to_ip(), self.port())
    }
}

impl SocketAddressV6 {
    /// Converts the address to a socket address.
    pub const fn to_socket(&self) -> SocketAddress {
        SocketAddress::new(self.ip().to_ip(), self.port())
    }
}

impl SocketAddress {
    /// Converts the address to an optional IPv4 socket address.
    pub const fn to_v4(&self) -> Option<SocketAddressV4> {
        if let Some(v4) = self.ip().to_v4() {
            Some(SocketAddressV4::new(v4, self.port()))
        } else {
            None
        }
    }

    /// Converts the address to an optional IPv6 socket address.
    pub const fn to_v6(&self) -> Option<SocketAddressV6> {
        if let Some(v6) = self.ip().to_v6() {
            Some(SocketAddressV6::new(v6, self.port()))
        } else {
            None
        }
    }

    /// Converts the socket address to an authority.
    pub fn to_authority(&self) -> Authority {
        Authority::new(self.ip().to_host(), self.port())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Authority, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6,
    };

    #[test]
    fn v4_to_socket() {
        let result: SocketAddress = SocketAddressV4::new(IPv4Address::LOCALHOST, 80).to_socket();
        let expected: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn v6_to_socket() {
        let result: SocketAddress = SocketAddressV6::new(IPv6Address::LOCALHOST, 80).to_socket();
        let expected: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn socket_to_v4() {
        let socket: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        let result: Option<SocketAddressV4> = socket.to_v4();
        let expected: Option<SocketAddressV4> =
            Some(SocketAddressV4::new(IPv4Address::LOCALHOST, 80));
        assert_eq!(result, expected);

        let socket: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
        let result: Option<SocketAddressV4> = socket.to_v4();
        let expected: Option<SocketAddressV4> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn socket_to_v6() {
        let socket: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        let result: Option<SocketAddressV6> = socket.to_v6();
        let expected: Option<SocketAddressV6> = None;
        assert_eq!(result, expected);

        let socket: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
        let result: Option<SocketAddressV6> = socket.to_v6();
        let expected: Option<SocketAddressV6> =
            Some(SocketAddressV6::new(IPv6Address::LOCALHOST, 80));
        assert_eq!(result, expected);
    }

    #[test]
    fn socket_to_authority() {
        let socket: SocketAddress = IPv4Address::LOCALHOST.to_socket(80).to_socket();
        let result: Authority = socket.to_authority();
        let expected: Authority = Authority::new(IPv4Address::LOCALHOST, 80);
        assert_eq!(result, expected);
    }
}
