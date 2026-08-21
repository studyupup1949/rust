use std::str::FromStr;

use crate::{Domain, Host, IPAddress};

impl FromStr for Host {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ip) = IPAddress::from_str(s) {
            Ok(ip.into())
        } else if let Ok(domain) = Domain::from_str(s) {
            Ok(domain.into())
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
            ("localhost", Ok(Domain::localhost().into())),
            ("LocalHost", Ok(Domain::localhost().into())),
            ("127.0.0.1", Ok(IPv4Address::LOCALHOST.into())),
            ("::1", Ok(IPv6Address::LOCALHOST.into())),
        ];
        for (s, expected) in test_cases {
            let result: Result<Host, ()> = Host::from_str(*s);
            assert_eq!(result, *expected);
        }
    }
}
