use crate::{SocketAddress, SocketAddressV4, SocketAddressV6};
use std::fmt::{Debug, Display, Formatter};

impl Debug for SocketAddressV4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for SocketAddressV4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_std(), f)
    }
}

impl Debug for SocketAddressV6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for SocketAddressV6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_std(), f)
    }
}

impl Debug for SocketAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for SocketAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_std(), f)
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6};

    #[test]
    fn v4() {
        let test_cases: &[(SocketAddressV4, &str)] = &[(IPv4Address::LOCALHOST.to_socket(80), "127.0.0.1:80")];

        for (socket, expected) in test_cases {
            let result: String = socket.to_string();
            assert_eq!(result, *expected, "socket={:?}", socket);
        }
    }

    #[test]
    fn v6() {
        let test_cases: &[(SocketAddressV6, &str)] = &[(IPv6Address::LOCALHOST.to_socket(80), "[::1]:80")];

        for (socket, expected) in test_cases {
            let result: String = socket.to_string();
            assert_eq!(result, *expected, "socket={:?}", socket);
        }
    }

    #[test]
    fn socket() {
        let test_cases: &[(SocketAddress, &str)] = &[
            (IPv4Address::LOCALHOST.to_socket(80).to_socket(), "127.0.0.1:80"),
            (IPv6Address::LOCALHOST.to_socket(80).to_socket(), "[::1]:80"),
        ];

        for (socket, expected) in test_cases {
            let result: String = socket.to_string();
            assert_eq!(result, *expected, "socket={:?}", socket);
        }
    }
}
