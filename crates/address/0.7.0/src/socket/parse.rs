use std::str::FromStr;

use crate::{util, IPv4Address, IPv6Address};
use crate::{SocketAddress, SocketAddressV4, SocketAddressV6};

impl FromStr for SocketAddressV4 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = util::extract_port(s)?;
        let ip: IPv4Address = IPv4Address::from_str(s)?;
        Ok(SocketAddressV4::new(ip, port))
    }
}

impl FromStr for SocketAddressV6 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = util::extract_port(s)?;
        if !s.is_empty() && s.as_bytes()[0] == b'[' {
            if s.as_bytes()[s.len() - 1] != b']' {
                Err(())
            } else {
                let s: &str = &s[1..(s.len() - 1)];
                let ip: IPv6Address = IPv6Address::from_str(s)?;
                Ok(SocketAddressV6::new(ip, port))
            }
        } else {
            Err(())
        }
    }
}

impl FromStr for SocketAddress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = util::extract_port(s)?;
        if !s.is_empty() && s.as_bytes()[0] == b'[' {
            if s.as_bytes()[s.len() - 1] != b']' {
                Err(())
            } else {
                let s: &str = &s[1..(s.len() - 1)];
                let ip: IPv6Address = IPv6Address::from_str(s)?;
                Ok(SocketAddress::new(ip.to_ip(), port))
            }
        } else {
            let ip: IPv4Address = IPv4Address::from_str(s)?;
            Ok(SocketAddress::new(ip.to_ip(), port))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6};

    #[test]
    fn v4() {
        let test_cases: &[(&str, Result<SocketAddressV4, ()>)] = &[
            ("", Err(())),
            (":80", Err(())),
            ("::1:80", Err(())),
            ("[127.0.0.1]:80", Err(())),
            (
                "127.0.0.1:80",
                Ok(SocketAddressV4::new(IPv4Address::LOCALHOST, 80)),
            ),
        ];
        for (s, expected) in test_cases {
            let result: Result<SocketAddressV4, ()> = SocketAddressV4::from_str(*s);
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn v6() {
        let test_cases: &[(&str, Result<SocketAddressV6, ()>)] = &[
            ("", Err(())),
            (":80", Err(())),
            ("[127.0.0.1]:80", Err(())),
            ("::1:80", Err(())),
            (
                "[::1]:80",
                Ok(SocketAddressV6::new(IPv6Address::LOCALHOST, 80)),
            ),
        ];
        for (s, expected) in test_cases {
            let result: Result<SocketAddressV6, ()> = SocketAddressV6::from_str(*s);
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn ip() {
        let test_cases: &[(&str, Result<SocketAddress, ()>)] = &[
            ("", Err(())),
            (":80", Err(())),
            ("[127.0.0.1]:80", Err(())),
            ("::1:80", Err(())),
            (
                "127.0.0.1:80",
                Ok(SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80)),
            ),
            (
                "[::1]:80",
                Ok(SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80)),
            ),
        ];
        for (s, expected) in test_cases {
            let result: Result<SocketAddress, ()> = SocketAddress::from_str(*s);
            assert_eq!(result, *expected);
        }
    }
}
