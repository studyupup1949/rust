use std::str::FromStr;

use crate::{util, Authority, Host, IPv6Address};

impl FromStr for Authority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = util::extract_port(s)?;
        if !s.as_bytes().is_empty() && s.as_bytes()[0] == b'[' {
            if s.as_bytes()[s.len() - 1] != b']' {
                Err(())
            } else {
                let s: &str = &s[1..(s.len() - 1)];
                let ip: IPv6Address = IPv6Address::from_str(s)?;
                Ok(Authority::new(ip, port))
            }
        } else {
            let host: Host = Host::from_str(s)?;
            Ok(Authority::new(host, port))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{Authority, Domain, IPv4Address, IPv6Address};

    #[test]
    fn authority() {
        let test_cases: &[(&str, Result<Authority, ()>)] = &[
            (
                "127.0.0.1:80",
                Ok(Authority::new(IPv4Address::LOCALHOST, 80)),
            ),
            ("[::1]:80", Ok(Authority::new(IPv6Address::LOCALHOST, 80))),
            ("localhost:80", Ok(Authority::new(Domain::localhost(), 80))),
        ];
        for (s, expected) in test_cases {
            let result: Result<Authority, ()> = Authority::from_str(*s);
            assert_eq!(result, *expected);
        }
    }
}
