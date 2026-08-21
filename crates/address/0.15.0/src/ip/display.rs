use std::fmt::{Display, Formatter};

use crate::{IPAddress, IPv4Address, IPv6Address};

impl Display for IPv4Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (a, b, c, d) = self.bytes();
        write!(f, "{}.{}.{}.{}", a, b, c, d)
    }
}

impl Display for IPv6Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_std())
    }
}

impl Display for IPAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V4(ip) => write!(f, "{}", ip),
            Self::V6(ip) => write!(f, "{}", ip),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn v4() {
        let test_cases: &[(IPv4Address, &str)] = &[
            (IPv4Address::UNSPECIFIED, "0.0.0.0"),
            (IPv4Address::LOCALHOST, "127.0.0.1"),
            (IPv4Address::BROADCAST, "255.255.255.255"),
        ];

        for (ip, expected) in test_cases {
            let result: String = ip.to_string();
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn v6() {
        let test_cases: &[(IPv6Address, &str)] = &[
            (IPv6Address::UNSPECIFIED, "::"),
            (IPv6Address::LOCALHOST, "::1"),
        ];

        for (ip, expected) in test_cases {
            let result: String = ip.to_string();
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn ip() {
        let test_cases: &[(IPAddress, &str)] = &[
            (IPv4Address::LOCALHOST.to_ip(), "127.0.0.1"),
            (IPv6Address::LOCALHOST.to_ip(), "::1"),
        ];

        for (ip, expected) in test_cases {
            let result: String = ip.to_string();
            assert_eq!(result, *expected);
        }
    }
}
