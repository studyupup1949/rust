use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use crate::ParseError::{InvalidIPAddress, InvalidIPv4Address, InvalidIPv6Address};
use crate::{IPAddress, IPv4Address, IPv6Address, ParseError};

impl FromStr for IPv4Address {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ipv4Addr::from_str(s)
            .map_err(|_| InvalidIPv4Address)?
            .into())
    }
}

impl FromStr for IPv6Address {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ipv6Addr::from_str(s)
            .map_err(|_| InvalidIPv6Address)?
            .into())
    }
}

impl FromStr for IPAddress {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ip) = IPv4Address::from_str(s) {
            Ok(ip.to_ip())
        } else if let Ok(ip) = IPv6Address::from_str(s) {
            Ok(ip.to_ip())
        } else {
            Err(InvalidIPAddress)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ParseError::{InvalidIPAddress, InvalidIPv4Address, InvalidIPv6Address};
    use crate::{IPAddress, IPv4Address, IPv6Address, ParseError};

    #[test]
    fn v4() {
        let test_cases: &[(&str, Result<IPv4Address, ParseError>)] = &[
            ("", Err(InvalidIPv4Address)),
            ("0.0.0.0", Ok(IPv4Address::UNSPECIFIED)),
            ("127.0.0.1", Ok(IPv4Address::LOCALHOST)),
            ("255.255.255.255", Ok(IPv4Address::BROADCAST)),
        ];

        for (input, expected) in test_cases {
            let result: Result<IPv4Address, ParseError> = IPv4Address::from_str(input);
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn v6() {
        let test_cases: &[(&str, Result<IPv6Address, ParseError>)] = &[
            ("", Err(InvalidIPv6Address)),
            ("::", Ok(IPv6Address::UNSPECIFIED)),
            ("::1", Ok(IPv6Address::LOCALHOST)),
        ];

        for (input, expected) in test_cases {
            let result: Result<IPv6Address, ParseError> = IPv6Address::from_str(input);
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn ip() {
        let test_cases: &[(&str, Result<IPAddress, ParseError>)] = &[
            ("", Err(InvalidIPAddress)),
            ("127.0.0.1", Ok(IPv4Address::LOCALHOST.to_ip())),
            ("::1", Ok(IPv6Address::LOCALHOST.to_ip())),
        ];

        for (input, expected) in test_cases {
            let result: Result<IPAddress, ParseError> = IPAddress::from_str(input);
            assert_eq!(result, *expected);
        }
    }
}
