use std::str::FromStr;

use crate::features::parse::util::extract_port;
use crate::{IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6};

impl FromStr for SocketAddressV4 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = extract_port(s)?;
        let v4: IPv4Address = IPv4Address::from_str(s)?;
        Ok(SocketAddressV4::new(v4, port))
    }
}

impl FromStr for SocketAddressV6 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = extract_port(s)?;
        if !s.is_empty() && s.as_bytes()[0] == b'[' && s.as_bytes()[s.len() - 1] == b']' {
            let v6: IPv6Address = IPv6Address::from_str(&s[1..(s.len() - 1)])?;
            Ok(SocketAddressV6::new(v6, port))
        } else {
            Err(())
        }
    }
}

impl FromStr for SocketAddress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = extract_port(s)?;
        if !s.is_empty() && s.as_bytes()[0] == b'[' && s.as_bytes()[s.len() - 1] == b']' {
            let v6: IPv6Address = IPv6Address::from_str(&s[1..(s.len() - 1)])?;
            Ok(SocketAddress::new(v6.to_ip(), port))
        } else {
            let v4: IPv4Address = IPv4Address::from_str(s)?;
            Ok(SocketAddress::new(v4.to_ip(), port))
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
            ("127.0.0.1", Err(())),
            ("127.0.0.1:", Err(())),
            ("127.0.0.1:x", Err(())),
            ("80", Err(())),
            (":80", Err(())),
            ("x:80", Err(())),
            ("[::1]:80", Err(())),
            ("127.0.0.1:80", Ok(IPv4Address::LOCALHOST.to_socket(80))),
        ];
        for (s, expected) in test_cases {
            let result = SocketAddressV4::from_str(*s);
            assert_eq!(result, *expected, "{}", s);
        }
    }

    #[test]
    fn v6() {
        let test_cases: &[(&str, Result<SocketAddressV6, ()>)] = &[
            ("", Err(())),
            ("::1", Err(())),
            ("[::1]", Err(())),
            ("[::1]:", Err(())),
            ("[::1]:x", Err(())),
            ("80", Err(())),
            (":80", Err(())),
            ("x:80", Err(())),
            ("127.0.0.1:80", Err(())),
            ("[::1]:80", Ok(IPv6Address::LOCALHOST.to_socket(80))),
        ];
        for (s, expected) in test_cases {
            let result = SocketAddressV6::from_str(*s);
            assert_eq!(result, *expected, "{}", s);
        }
    }

    #[test]
    fn socket() {
        let test_cases: &[(&str, Result<SocketAddress, ()>)] = &[
            ("", Err(())),
            ("::1", Err(())),
            ("127.0.0.1", Err(())),
            ("127.0.0.1:", Err(())),
            ("127.0.0.1:x", Err(())),
            ("[::1]", Err(())),
            ("[::1]:", Err(())),
            ("[::1]:x", Err(())),
            ("80", Err(())),
            (":80", Err(())),
            ("x:80", Err(())),
            (
                "127.0.0.1:80",
                Ok(IPv4Address::LOCALHOST.to_socket(80).to_socket()),
            ),
            (
                "[::1]:80",
                Ok(IPv6Address::LOCALHOST.to_socket(80).to_socket()),
            ),
        ];
        for (s, expected) in test_cases {
            let result = SocketAddress::from_str(*s);
            assert_eq!(result, *expected, "{}", s);
        }
    }
}
