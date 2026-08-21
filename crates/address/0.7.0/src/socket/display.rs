use std::fmt::{Display, Formatter};

use crate::{IPAddress, SocketAddress, SocketAddressV4, SocketAddressV6};

impl Display for SocketAddressV4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}

impl Display for SocketAddressV6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]:{}", self.ip(), self.port())
    }
}

impl Display for SocketAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.ip() {
            IPAddress::V4(ip) => write!(f, "{}:{}", ip, self.port()),
            IPAddress::V6(ip) => write!(f, "[{}]:{}", ip, self.port()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6};

    #[test]
    fn v4() {
        let socket: SocketAddressV4 = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(socket.to_string(), "127.0.0.1:80");
    }

    #[test]
    fn v6() {
        let socket: SocketAddressV6 = (IPv6Address::LOCALHOST, 80).into();
        assert_eq!(socket.to_string(), "[::1]:80");
    }

    #[test]
    fn ip() {
        let socket: SocketAddress = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(socket.to_string(), "127.0.0.1:80");

        let socket: SocketAddress = (IPv6Address::LOCALHOST, 80).into();
        assert_eq!(socket.to_string(), "[::1]:80");
    }
}
