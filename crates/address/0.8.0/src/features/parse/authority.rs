use std::str::FromStr;

use crate::{Authority, Endpoint, SocketAddress};

impl FromStr for Authority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(socket) = SocketAddress::from_str(s) {
            Ok(socket.to_authority())
        } else if let Ok(endpoint) = Endpoint::from_str(s) {
            Ok(endpoint.to_authority())
        } else {
            Err(())
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
            ("", Err(())),
            ("localhost", Err(())),
            ("localhost:", Err(())),
            ("localhost:x", Err(())),
            ("80", Err(())),
            (":80", Err(())),
            ("$@#:80", Err(())),
            ("::1", Err(())),
            ("127.0.0.1", Err(())),
            ("127.0.0.1:", Err(())),
            ("127.0.0.1:x", Err(())),
            ("[::1]", Err(())),
            ("[::1]:", Err(())),
            ("[::1]:x", Err(())),
            ("80", Err(())),
            (":80", Err(())),
            (
                "localhost:80",
                Ok(Domain::localhost().to_endpoint(80).to_authority()),
            ),
            (
                "LocalHost:80",
                Ok(Domain::localhost().to_endpoint(80).to_authority()),
            ),
            (
                "127.0.0.1:80",
                Ok(IPv4Address::LOCALHOST
                    .to_socket(80)
                    .to_socket()
                    .to_authority()),
            ),
            (
                "[::1]:80",
                Ok(IPv6Address::LOCALHOST
                    .to_socket(80)
                    .to_socket()
                    .to_authority()),
            ),
        ];
        for (s, expected) in test_cases {
            let result = Authority::from_str(*s);
            assert_eq!(result, *expected, "{}", *s);
        }
    }
}
