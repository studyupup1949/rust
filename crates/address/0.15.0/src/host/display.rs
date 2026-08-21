use std::fmt::{Display, Formatter};

use crate::{Host, HostRef};

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ref())
    }
}

impl<'a> Display for HostRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(domain) => write!(f, "{}", domain),
            Self::Address(ip) => write!(f, "{}", ip),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, Host, IPv4Address, IPv6Address};

    #[test]
    fn display() {
        let test_cases: &[(Host, &str)] = &[
            (Domain::localhost().to_host(), "localhost"),
            (IPv4Address::LOCALHOST.to_host(), "127.0.0.1"),
            (IPv6Address::LOCALHOST.to_host(), "::1"),
        ];

        for (host, expected) in test_cases {
            let result: String = host.to_string();
            assert_eq!(result, *expected);
        }
    }
}
