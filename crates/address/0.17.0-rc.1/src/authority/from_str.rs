use std::str::FromStr;

use crate::parse_port;
use crate::ParseError::InvalidAuthority;
use crate::{Authority, Host, IPv6Address, ParseError};

impl FromStr for Authority {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = parse_port(s)?;
        if !s.is_empty() && s.as_bytes()[0] == b'[' && s.as_bytes()[s.len() - 1] == b']' {
            let s: &str = &s[1..(s.len() - 1)];
            let host: Host = IPv6Address::from_str(s)?.to_host();
            Ok(host.to_authority(port))
        } else {
            let host: Host = Host::from_str(s)?;
            if let Host::Address(ip) = host {
                if ip.is_v6() {
                    return Err(InvalidAuthority);
                }
            }
            Ok(host.to_authority(port))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ParseError::{InvalidAuthority, InvalidHost, InvalidPort};
    use crate::{Authority, Domain, IPv4Address, IPv6Address, ParseError};

    #[test]
    fn from_str() {
        let test_cases: &[(&str, Result<Authority, ParseError>)] = &[
            ("", Err(InvalidPort)),
            ("localhost:", Err(InvalidPort)),
            ("localhost:xx", Err(InvalidPort)),
            (":80", Err(InvalidHost)),
            (
                "127.0.0.1:80",
                Ok(IPv4Address::LOCALHOST.to_host().to_authority(80)),
            ),
            ("::1:80", Err(InvalidAuthority)),
            (
                "[::1]:80",
                Ok(IPv6Address::LOCALHOST.to_host().to_authority(80)),
            ),
            (
                "localhost:80",
                Ok(Domain::localhost().to_host().to_authority(80)),
            ),
        ];

        for (input, expected) in test_cases {
            let result: Result<Authority, ParseError> = Authority::from_str(input);
            assert_eq!(result, *expected, "input={}", *input);
        }
    }
}
