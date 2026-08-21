use std::str::FromStr;

use crate::ParseError::InvalidHost;
use crate::{Domain, Host, IPAddress, ParseError};

impl FromStr for Host {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ip) = IPAddress::from_str(s) {
            Ok(ip.to_host())
        } else if let Ok(domain) = Domain::from_str(s) {
            Ok(domain.to_host())
        } else {
            Err(InvalidHost)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ParseError::InvalidHost;
    use crate::{Domain, Host, IPv4Address, IPv6Address, ParseError};

    #[test]
    fn from_str() {
        let test_cases: &[(&str, Result<Host, ParseError>)] = &[
            ("", Err(InvalidHost)),
            ("localhost", Ok(Domain::localhost().to_host())),
            ("127.0.0.1", Ok(IPv4Address::LOCALHOST.to_host())),
            ("::1", Ok(IPv6Address::LOCALHOST.to_host())),
            ("[::1]", Err(InvalidHost)),
        ];

        for (input, expected) in test_cases {
            let result: Result<Host, ParseError> = Host::from_str(input);
            assert_eq!(result, *expected);
        }
    }
}
