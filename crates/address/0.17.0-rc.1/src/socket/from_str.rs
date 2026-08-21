use std::str::FromStr;

use crate::parse_port;
use crate::ParseError::InvalidSocketAddressV6;
use crate::{
    IPv4Address, IPv6Address, ParseError, SocketAddress, SocketAddressV4, SocketAddressV6,
};

impl FromStr for SocketAddressV4 {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = parse_port(s)?;
        let ip: IPv4Address = IPv4Address::from_str(s)?;
        Ok(SocketAddressV4::new(ip, port))
    }
}

impl FromStr for SocketAddressV6 {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = parse_port(s)?;
        if s.is_empty() || s.as_bytes()[0] != b'[' || s.as_bytes()[s.len() - 1] != b']' {
            Err(InvalidSocketAddressV6)
        } else {
            let s: &str = &s[1..s.len() - 1];
            let ip: IPv6Address = IPv6Address::from_str(s)?;
            Ok(SocketAddressV6::new(ip, port))
        }
    }
}

impl FromStr for SocketAddress {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = parse_port(s)?;
        if s.is_empty() || s.as_bytes()[0] != b'[' || s.as_bytes()[s.len() - 1] != b']' {
            let ip: IPv4Address = IPv4Address::from_str(s)?;
            Ok(ip.to_socket(port).to_socket())
        } else {
            let s: &str = &s[1..s.len() - 1];
            let ip: IPv6Address = IPv6Address::from_str(s)?;
            Ok(ip.to_socket(port).to_socket())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ParseError::{
        InvalidIPv4Address, InvalidIPv6Address, InvalidPort, InvalidSocketAddressV6,
    };
    use crate::{
        IPv4Address, IPv6Address, ParseError, SocketAddress, SocketAddressV4, SocketAddressV6,
    };

    #[test]
    fn v4() {
        let test_cases: &[(&str, Result<SocketAddressV4, ParseError>)] = &[
            ("", Err(InvalidPort)),
            ("127.0.0.1:", Err(InvalidPort)),
            ("127.0.0.1:xx", Err(InvalidPort)),
            (":80", Err(InvalidIPv4Address)),
            ("xx:80", Err(InvalidIPv4Address)),
            ("127.0.0.1:80", Ok(IPv4Address::LOCALHOST.to_socket(80))),
        ];

        for (input, expected) in test_cases {
            let result: Result<SocketAddressV4, ParseError> = SocketAddressV4::from_str(input);
            assert_eq!(result, *expected, "input={}", input);
        }
    }

    #[test]
    fn v6() {
        let test_cases: &[(&str, Result<SocketAddressV6, ParseError>)] = &[
            ("", Err(InvalidPort)),
            ("[::1]:", Err(InvalidPort)),
            ("[::1]:xx", Err(InvalidPort)),
            (":80", Err(InvalidSocketAddressV6)),
            ("xx:80", Err(InvalidSocketAddressV6)),
            ("[xx]:80", Err(InvalidIPv6Address)),
            ("[::1]:80", Ok(IPv6Address::LOCALHOST.to_socket(80))),
        ];

        for (input, expected) in test_cases {
            let result: Result<SocketAddressV6, ParseError> = SocketAddressV6::from_str(input);
            assert_eq!(result, *expected, "input={}", input);
        }
    }

    #[test]
    fn socket() {
        let test_cases: &[(&str, Result<SocketAddress, ParseError>)] = &[
            ("", Err(InvalidPort)),
            ("[::1]:", Err(InvalidPort)),
            ("[::1]:xx", Err(InvalidPort)),
            (":80", Err(InvalidIPv4Address)),
            ("xx:80", Err(InvalidIPv4Address)),
            ("::1:80", Err(InvalidIPv4Address)),
            ("[]:80", Err(InvalidIPv6Address)),
            ("[xx]:80", Err(InvalidIPv6Address)),
            (
                "[::1]:80",
                Ok(IPv6Address::LOCALHOST.to_socket(80).to_socket()),
            ),
        ];

        for (input, expected) in test_cases {
            let result: Result<SocketAddress, ParseError> = SocketAddress::from_str(input);
            assert_eq!(result, *expected, "input={}", input);
        }
    }
}
