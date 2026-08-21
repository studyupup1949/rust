use crate::ParseError::{InvalidIPAddress, InvalidIPv4Address, InvalidIPv6Address};
use crate::{IPAddress, IPv4Address, IPv6Address, ParseError};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

impl IPv4Address {
    //! Parse

    /// The maximum length of an IPv4 address string. (255.255.255.255)
    const MAX_STR_LEN: usize = 15;

    /// Parses an IPv4 address from the `address` bytes.
    pub(crate) fn parse(address: &[u8]) -> Result<Self, ParseError> {
        if address.len() > Self::MAX_STR_LEN {
            return Err(InvalidIPv4Address);
        }
        let address: &str = std::str::from_utf8(address).map_err(|_| InvalidIPv4Address)?;
        Self::from_str(address)
    }
}

impl FromStr for IPv4Address {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ipv4Addr::from_str(s).map_err(|_| InvalidIPv4Address)?.into())
    }
}

impl IPv6Address {
    //! Parse

    /// The maximum length of an IPv6 address string. (ffff:ffff:ffff:ffff:ffff:ffff:255.255.255.255)
    const MAX_STR_LEN: usize = 45;

    /// Parses an IPv6 address from the `address` bytes.
    pub(crate) fn parse(address: &[u8]) -> Result<Self, ParseError> {
        if address.len() > Self::MAX_STR_LEN {
            return Err(InvalidIPv6Address);
        }
        let address: &str = std::str::from_utf8(address).map_err(|_| InvalidIPv6Address)?;
        Self::from_str(address)
    }
}

impl FromStr for IPv6Address {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ipv6Addr::from_str(s).map_err(|_| InvalidIPv6Address)?.into())
    }
}

impl IPAddress {
    //! Parse

    /// Parses an IP address from the `address` bytes.
    pub(crate) fn parse(address: &[u8]) -> Result<Self, ParseError> {
        if let Ok(ip) = IPv4Address::parse(address) {
            Ok(ip.to_ip())
        } else if let Ok(ip) = IPv6Address::parse(address) {
            Ok(ip.to_ip())
        } else {
            Err(InvalidIPAddress)
        }
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
    use crate::ParseError::{InvalidIPAddress, InvalidIPv4Address, InvalidIPv6Address};
    use crate::{IPAddress, IPv4Address, IPv6Address, ParseError};
    use std::str::FromStr;

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
            assert_eq!(result, *expected, "input={}", input);

            let result: Result<IPv4Address, ParseError> = IPv4Address::parse(input.as_bytes());
            assert_eq!(result, *expected, "input={}", input);
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
            assert_eq!(result, *expected, "input={}", input);

            let result: Result<IPv6Address, ParseError> = IPv6Address::parse(input.as_bytes());
            assert_eq!(result, *expected, "input={}", input);
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
            assert_eq!(result, *expected, "input={}", input);

            let result: Result<IPAddress, ParseError> = IPAddress::parse(input.as_bytes());
            assert_eq!(result, *expected, "input={}", input);
        }
    }
}
