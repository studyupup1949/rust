use crate::{Authority, IPAddress, SocketAddress, SocketAddressV4, SocketAddressV6};

impl SocketAddressV4 {
    //! Conversions

    /// Converts the IPv4 socket address to a socket address.
    pub const fn to_socket(&self) -> SocketAddress {
        SocketAddress::new(self.ip().to_ip(), self.port())
    }
}

impl SocketAddressV6 {
    //! Conversions

    /// Converts the IPv6 socket address to a socket address.
    pub const fn to_socket(&self) -> SocketAddress {
        SocketAddress::new(self.ip().to_ip(), self.port())
    }
}

impl SocketAddress {
    //! Conversions

    /// Converts the socket address to an optional IPv4 socket address.
    pub const fn to_v4(&self) -> Option<SocketAddressV4> {
        if let IPAddress::V4(v4) = self.ip() {
            Some(SocketAddressV4::new(v4, self.port()))
        } else {
            None
        }
    }

    /// Converts the socket address to an optional IPv6 socket address.
    pub const fn to_v6(&self) -> Option<SocketAddressV6> {
        if let IPAddress::V6(v6) = self.ip() {
            Some(SocketAddressV6::new(v6, self.port()))
        } else {
            None
        }
    }

    /// Converts the socket address to an authority.
    pub const fn to_authority(&self) -> Authority {
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
        let socket: SocketAddressV4 = IPv4Address::LOCALHOST.to_socket(80);
        let result: SocketAddress = socket.to_socket();
        let expected: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn v6_to_socket() {
        let socket: SocketAddressV6 = IPv6Address::LOCALHOST.to_socket(80);
        let result: SocketAddress = socket.to_socket();
        let expected: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn socket_to_v4() {
        let socket: SocketAddress = IPv4Address::LOCALHOST.to_ip().to_socket(80);
        let result: Option<SocketAddressV4> = socket.to_v4();
        let expected: Option<SocketAddressV4> =
            Some(SocketAddressV4::new(IPv4Address::LOCALHOST, 80));
        assert_eq!(result, expected);

        let socket: SocketAddress = IPv6Address::LOCALHOST.to_ip().to_socket(80);
        let result: Option<SocketAddressV4> = socket.to_v4();
        let expected: Option<SocketAddressV4> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn socket_to_v6() {
        let socket: SocketAddress = IPv4Address::LOCALHOST.to_ip().to_socket(80);
        let result: Option<SocketAddressV6> = socket.to_v6();
        let expected: Option<SocketAddressV6> = None;
        assert_eq!(result, expected);

        let socket: SocketAddress = IPv6Address::LOCALHOST.to_ip().to_socket(80);
        let result: Option<SocketAddressV6> = socket.to_v6();
        let expected: Option<SocketAddressV6> =
            Some(SocketAddressV6::new(IPv6Address::LOCALHOST, 80));
        assert_eq!(result, expected);
    }

    #[test]
    fn socket_to_authority() {
        let socket: SocketAddress = IPv4Address::LOCALHOST.to_socket(80).to_socket();
        let result: Authority = socket.to_authority();
        let expected: Authority = Authority::new(IPv4Address::LOCALHOST.to_host(), 80);
        assert_eq!(result, expected);
    }
}
