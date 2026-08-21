use std::str::FromStr;

use crate::{Domain, Host, IPAddress};

impl FromStr for Host {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ip) = IPAddress::from_str(s) {
            Ok(ip.to_host())
        } else if let Ok(domain) = Domain::from_str(s) {
            Ok(domain.to_host())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{Domain, Host, IPv4Address, IPv6Address};

    #[test]
    fn host() {
        let test_cases: &[(&str, Result<Host, ()>)] = &[
            ("", Err(())),
            ("[::1]", Err(())),
            ("127.0.0.1", Ok(IPv4Address::LOCALHOST.to_host())),
            ("::1", Ok(IPv6Address::LOCALHOST.to_host())),
            ("localhost", Ok(Domain::localhost().to_host())),
        ];
        for (s, expected) in test_cases {
            let result = Host::from_str(*s);
            assert_eq!(result, *expected, "{}", *s);
        }
    }
}
