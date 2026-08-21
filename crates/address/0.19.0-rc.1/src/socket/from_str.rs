use crate::ParseError::{InvalidSocketAddress, InvalidSocketAddressV6};
use crate::{parse_port, strip_brackets};
use crate::{
    IPv4Address, IPv6Address, ParseError, SocketAddress, SocketAddressV4, SocketAddressV6,
};
use std::str::FromStr;

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
        let s: &str = strip_brackets(s).ok_or(InvalidSocketAddressV6)?;
        let ip: IPv6Address = IPv6Address::from_str(s)?;
        Ok(SocketAddressV6::new(ip, port))
    }
}

impl FromStr for SocketAddress {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = parse_port(s)?;
        if let Some(s) = strip_brackets(s) {
            let ip: IPv6Address = IPv6Address::from_str(s)?;
            Ok(ip.to_socket(port).to_socket())
        } else {
            let ip: IPv4Address = IPv4Address::from_str(s).map_err(|_| InvalidSocketAddress)?;
            Ok(ip.to_socket(port).to_socket())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ParseError::{
        InvalidIPv4Address, InvalidIPv6Address, InvalidPort, InvalidSocketAddress,
        InvalidSocketAddressV6,
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
            (
                "127.0.0.1:65535",
                Ok(IPv4Address::LOCALHOST.to_socket(65535)),
            ),
            ("127.0.0.1:65536", Err(InvalidPort)),
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
            (":80", Err(InvalidSocketAddress)),
            ("xx:80", Err(InvalidSocketAddress)),
            ("::1:80", Err(InvalidSocketAddress)),
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
