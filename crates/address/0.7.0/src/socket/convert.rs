use crate::{Authority, SocketAddress, SocketAddressV4, SocketAddressV6};

impl SocketAddressV4 {
    //! Conversions

    /// Converts the address to a socket address.
    pub fn to_socket(&self) -> SocketAddress {
        SocketAddress::new(self.ip().to_ip(), self.port())
    }
}

impl SocketAddressV6 {
    //! Conversions

    /// Converts the address to a socket address.
    pub fn to_socket(&self) -> SocketAddress {
        SocketAddress::new(self.ip().to_ip(), self.port())
    }
}

impl SocketAddress {
    //! Conversions

    /// Converts the address to an authority.
    pub fn to_authority(&self) -> Authority {
        Authority::new(self.ip().to_host(), self.port())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, IPv4Address, IPv6Address, SocketAddress};

    #[test]
    fn v4() {
        let result: SocketAddress = IPv4Address::LOCALHOST.to_socket(80).to_socket();
        let expected: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn v6() {
        let result: SocketAddress = IPv6Address::LOCALHOST.to_socket(80).to_socket();
        let expected: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn socket() {
        let result: Authority = IPv4Address::LOCALHOST.to_ip().to_socket(80).to_authority();
        let expected: Authority = Authority::new(IPv4Address::LOCALHOST.to_host(), 80);
        assert_eq!(result, expected);
    }
}
