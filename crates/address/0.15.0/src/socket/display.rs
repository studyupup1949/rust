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
        let socket: SocketAddressV4 = IPv4Address::LOCALHOST.to_socket(80);

        let result: String = socket.to_string();
        let expected: &str = "127.0.0.1:80";
        assert_eq!(result, expected);
    }

    #[test]
    fn v6() {
        let socket: SocketAddressV6 = IPv6Address::LOCALHOST.to_socket(80);

        let result: String = socket.to_string();
        let expected: &str = "[::1]:80";
        assert_eq!(result, expected);
    }

    #[test]
    fn socket() {
        let socket: SocketAddress = IPv4Address::LOCALHOST.to_socket(80).to_socket();
        let result: String = socket.to_string();
        let expected: &str = "127.0.0.1:80";
        assert_eq!(result, expected);

        let socket: SocketAddress = IPv6Address::LOCALHOST.to_socket(80).to_socket();
        let result: String = socket.to_string();
        let expected: &str = "[::1]:80";
        assert_eq!(result, expected);
    }
}
