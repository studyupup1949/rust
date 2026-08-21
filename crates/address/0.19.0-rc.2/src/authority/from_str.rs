use crate::ParseError::InvalidAuthority;
use crate::{parse_port, strip_brackets};
use crate::{Authority, Host, IPv6Address, ParseError};
use std::str::FromStr;

impl FromStr for Authority {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = parse_port(s)?;
        if let Some(s) = strip_brackets(s) {
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
    use crate::ParseError::{InvalidAuthority, InvalidHost, InvalidPort};
    use crate::{Authority, Domain, IPv4Address, IPv6Address, ParseError};
    use std::str::FromStr;

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
