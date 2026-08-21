use crate::{
    Host, HostRef, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4,
    SocketAddressV6,
};

impl IPv4Address {
    //! Conversions

    /// Converts the address to an IPv6 compatible address. (::a.b.c.d)
    pub const fn to_v6_compatible(&self) -> IPv6Address {
        let (a, b, c, d) = self.bytes();
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, a, b, c, d])
    }

    /// Converts the address to an IPv6 mapped address. (::ffff:a.b.c.d)
    pub const fn to_v6_mapped(&self) -> IPv6Address {
        let (a, b, c, d) = self.bytes();
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, a, b, c, d])
    }

    /// Converts the address to an IP address.
    pub const fn to_ip(&self) -> IPAddress {
        IPAddress::V4(*self)
    }

    /// Converts the address to a socket address with the `port`.
    pub const fn to_socket(&self, port: u16) -> SocketAddressV4 {
        SocketAddressV4::new(*self, port)
    }

    /// Converts the address to a host.
    pub const fn to_host(&self) -> Host {
        Host::Address(self.to_ip())
    }

    /// Converts the address to a host reference.
    pub const fn to_host_ref(&self) -> HostRef<'_> {
        HostRef::Address(self.to_ip())
    }
}

impl IPv6Address {
    //! Conversions

    /// Converts the address to an optional IPv4 address. Returns `None` if the address is not an
    /// IPv4 compatible address (::a.b.c.d) or an IPv4 mapped address (::ffff:a.b.c.d).
    pub const fn to_v4(&self) -> Option<IPv4Address> {
        match self.address() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, a, b, c, d] => {
                Some(IPv4Address::new([a, b, c, d]))
            }
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, a, b, c, d] => {
                Some(IPv4Address::new([a, b, c, d]))
            }
            _ => None,
        }
    }

    /// Converts the address to an IP address.
    pub const fn to_ip(&self) -> IPAddress {
        IPAddress::V6(*self)
    }

    /// Converts the address to a socket address with the `port`.
    pub const fn to_socket(&self, port: u16) -> SocketAddressV6 {
        SocketAddressV6::new(*self, port)
    }

    /// Converts the address to a host.
    pub const fn to_host(&self) -> Host {
        Host::Address(self.to_ip())
    }

    /// Converts the address to a host reference.
    pub const fn to_host_ref(&self) -> HostRef<'_> {
        HostRef::Address(self.to_ip())
    }
}

impl IPAddress {
    //! Conversions

    /// Converts the address to an optional IPv4 address.
    pub const fn to_v4(&self) -> Option<IPv4Address> {
        if let Self::V4(ip) = self {
            Some(*ip)
        } else {
            None
        }
    }

    /// Converts the address to an optional IPv6 address.
    pub const fn to_v6(&self) -> Option<IPv6Address> {
        if let Self::V6(ip) = self {
            Some(*ip)
        } else {
            None
        }
    }

    /// Converts the address to a socket address with the `port`.
    pub const fn to_socket(&self, port: u16) -> SocketAddress {
        SocketAddress::new(*self, port)
    }

    /// Converts the address to a host.
    pub const fn to_host(&self) -> Host {
        Host::Address(*self)
    }

    /// Converts the address to a host reference.
    pub const fn to_host_ref(&self) -> HostRef<'_> {
        HostRef::Address(*self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Host, HostRef, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4,
        SocketAddressV6,
    };

    #[test]
    fn v4_to_v6() {
        let ip: IPv4Address = IPv4Address::LOCALHOST;
        let result: IPv6Address = ip.to_v6_compatible();
        let expected: IPv6Address = IPv6Address::from([0, 0, 0, 0, 0, 0, 0x7F00, 1]);
        assert_eq!(result, expected);

        let ip: IPv4Address = IPv4Address::LOCALHOST;
        let result: IPv6Address = ip.to_v6_mapped();
        let expected: IPv6Address = IPv6Address::from([0, 0, 0, 0, 0, 0xFFFF, 0x7F00, 1]);
        assert_eq!(result, expected);
    }

    #[test]
    fn v4_to_ip() {
        let ip: IPv4Address = IPv4Address::LOCALHOST;
        let result: IPAddress = ip.to_ip();
        let expected: IPAddress = IPAddress::V4(IPv4Address::LOCALHOST);
        assert_eq!(result, expected);
    }

    #[test]
    fn v4_to_socket() {
        let ip: IPv4Address = IPv4Address::LOCALHOST;
        let result: SocketAddressV4 = ip.to_socket(80);
        let expected: SocketAddressV4 = SocketAddressV4::new(IPv4Address::LOCALHOST, 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn v4_to_host() {
        let ip: IPv4Address = IPv4Address::LOCALHOST;
        let result: Host = ip.to_host();
        let expected: Host = Host::Address(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);

        let result: HostRef = ip.to_host_ref();
        let expected: HostRef = HostRef::Address(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }

    #[test]
    fn v6_to_v4() {
        let ip: IPv6Address = IPv6Address::from([0, 0, 0, 0, 0, 0, 0x7F00, 1]);
        let result: Option<IPv4Address> = ip.to_v4();
        let expected: Option<IPv4Address> = Some(IPv4Address::LOCALHOST);
        assert_eq!(result, expected);

        let ip: IPv6Address = IPv6Address::from([0, 0, 0, 0, 0, 0xFFFF, 0x7F00, 1]);
        let result: Option<IPv4Address> = ip.to_v4();
        let expected: Option<IPv4Address> = Some(IPv4Address::LOCALHOST);
        assert_eq!(result, expected);

        let ip: IPv6Address = IPv6Address::from([1, 0, 0, 0, 0, 0, 0, 0]);
        let result: Option<IPv4Address> = ip.to_v4();
        let expected: Option<IPv4Address> = None;
        assert_eq!(result, expected);

        let ip: IPv6Address = IPv6Address::from([0, 0, 0, 0, 0, 1, 0, 0]);
        let result: Option<IPv4Address> = ip.to_v4();
        let expected: Option<IPv4Address> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn v6_to_ip() {
        let ip: IPv6Address = IPv6Address::LOCALHOST;
        let result: IPAddress = ip.to_ip();
        let expected: IPAddress = IPAddress::V6(IPv6Address::LOCALHOST);
        assert_eq!(result, expected);
    }

    #[test]
    fn v6_to_socket() {
        let ip: IPv6Address = IPv6Address::LOCALHOST;
        let result: SocketAddressV6 = ip.to_socket(80);
        let expected: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn v6_to_host() {
        let ip: IPv6Address = IPv6Address::LOCALHOST;

        let result: Host = ip.to_host();
        let expected: Host = Host::Address(IPv6Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);

        let result: HostRef = ip.to_host_ref();
        let expected: HostRef = HostRef::Address(IPv6Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }

    #[test]
    fn ip_to_v4() {
        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();

        let result: Option<IPv4Address> = ip.to_v4();
        let expected: Option<IPv4Address> = Some(IPv4Address::LOCALHOST);
        assert_eq!(result, expected);

        let ip: IPAddress = IPv6Address::LOCALHOST.to_ip();
        let result: Option<IPv4Address> = ip.to_v4();
        let expected: Option<IPv4Address> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn ip_to_v6() {
        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();
        let result: Option<IPv6Address> = ip.to_v6();
        let expected: Option<IPv6Address> = None;
        assert_eq!(result, expected);

        let ip: IPAddress = IPv6Address::LOCALHOST.to_ip();
        let result: Option<IPv6Address> = ip.to_v6();
        let expected: Option<IPv6Address> = Some(IPv6Address::LOCALHOST);
        assert_eq!(result, expected);
    }

    #[test]
    fn ip_to_socket() {
        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();
        let result: SocketAddress = ip.to_socket(80);
        let expected: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ip_to_host() {
        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();

        let result: Host = ip.to_host();
        let expected: Host = Host::Address(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);

        let result: HostRef = ip.to_host_ref();
        let expected: HostRef = HostRef::Address(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }
}
