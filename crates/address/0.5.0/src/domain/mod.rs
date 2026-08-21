use std::convert::TryFrom;
use std::fmt::{Display, Error, Formatter};

use crate::{Authority, Endpoint, Host};

/// Represents a domain.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Domain<'a> {
    /// The name.
    name: &'a str,
}

impl<'a> Domain<'a> {
    //! Special Domains

    /// The localhost domain.
    pub const LOCALHOST: Domain<'static> = Domain { name: "localhost" };
}

impl<'a> Domain<'a> {
    //! Validation

    /// The maximum length of a domain label.
    pub const MAX_LABEL_LENGTH: usize = 63;

    /// The maximum length of a domain name.
    pub const MAX_NAME_LENGTH: usize = 253;

    /// Checks if the char is valid. (not including dots or hyphens)
    fn is_valid_char(c: u8, ignore_case: bool) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_digit() || (ignore_case && c.is_ascii_uppercase())
    }

    /// Checks if the label is valid.
    pub fn is_valid_label(label: &[u8], ignore_case: bool) -> bool {
        if label.is_empty() || label.len() > Domain::MAX_LABEL_LENGTH {
            false
        } else {
            for (i, c) in label.iter().enumerate() {
                if !Domain::is_valid_char(*c, ignore_case) {
                    if *c == b'-' {
                        if i == 0 || label[i - 1] == b'-' || i == label.len() - 1 {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
            return true;
        }
    }

    /// Checks if the domain is valid.
    pub fn is_valid_domain(domain: &[u8], ignore_case: bool) -> bool {
        if domain.is_empty() || domain.len() > Domain::MAX_NAME_LENGTH {
            false
        } else {
            let mut rem: &[u8] = domain;
            loop {
                match rem.iter().position(|c| *c == b'.') {
                    Some(dot) => {
                        if !Domain::is_valid_label(&rem[..dot], ignore_case) {
                            return false;
                        }
                        rem = &rem[dot + 1..];
                    }
                    None => return Domain::is_valid_label(rem, ignore_case),
                }
            }
        }
    }
}

impl<'a> Domain<'a> {
    //! Conversions

    /// Converts the domain to an endpoint with the port.
    pub const fn to_endpoint(&self, port: u16) -> Endpoint {
        Endpoint::new(*self, port)
    }

    /// Converts the domain to a host.
    pub const fn to_host(&self) -> Host {
        Host::Name(*self)
    }

    /// Converts the domain to an authority with the port.
    pub const fn to_authority(&self, port: u16) -> Authority {
        Authority::Name(self.to_endpoint(port))
    }
}

impl<'a> TryFrom<&'a [u8]> for Domain<'a> {
    type Error = ();

    fn try_from(s: &'a [u8]) -> Result<Self, Self::Error> {
        if Domain::is_valid_domain(s, false) {
            let name: &'a str = unsafe { std::str::from_utf8_unchecked(s) };
            Ok(Domain { name })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<&'a str> for Domain<'a> {
    type Error = ();

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if Domain::is_valid_domain(s.as_bytes(), false) {
            Ok(Domain { name: s })
        } else {
            Err(())
        }
    }
}

impl<'a> Domain<'a> {
    //! Properties

    /// Gets the name.
    pub fn name(&self) -> &str {
        self.name
    }
}

impl<'a> Display for Domain<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod validation_tests {
    use crate::Domain;

    #[test]
    fn is_valid_label_empty() {
        assert_eq!(Domain::is_valid_label("".as_bytes(), false), false);
    }

    #[test]
    fn is_valid_label_length() {
        let label: String = (0..Domain::MAX_LABEL_LENGTH).map(|_| "a").collect();
        assert_eq!(Domain::is_valid_label(label.as_bytes(), false), true);

        let label: String = (0..Domain::MAX_LABEL_LENGTH + 1).map(|_| "a").collect();
        assert_eq!(Domain::is_valid_label(label.as_bytes(), false), false);
    }

    #[test]
    fn is_valid_label_invalid_chars() {
        "/:@[`{"
            .as_bytes()
            .iter()
            .map(|c| String::from_utf8(vec![*c]).unwrap())
            .for_each(|s| assert_eq!(Domain::is_valid_label(s.as_bytes(), false), false));
    }

    #[test]
    fn is_valid_label_valid_chars() {
        "az09"
            .as_bytes()
            .iter()
            .map(|c| String::from_utf8(vec![*c]).unwrap())
            .for_each(|s| assert_eq!(Domain::is_valid_label(s.as_bytes(), false), true));
        "AZ".as_bytes()
            .iter()
            .map(|c| String::from_utf8(vec![*c]).unwrap())
            .for_each(|s| assert_eq!(Domain::is_valid_label(s.as_bytes(), false), false));
        "azAZ09"
            .as_bytes()
            .iter()
            .map(|c| String::from_utf8(vec![*c]).unwrap())
            .for_each(|s| assert_eq!(Domain::is_valid_label(s.as_bytes(), true), true));
    }

    #[test]
    fn is_valid_label_hypens() {
        [
            ("a", true),
            ("-", false),
            ("a-", false),
            ("-a", false),
            ("a--b", false),
            ("a-b", true),
        ]
        .iter()
        .for_each(|t| {
            assert_eq!(
                Domain::is_valid_label(t.0.as_bytes(), false),
                t.1,
                "{}",
                t.0
            )
        })
    }

    #[test]
    fn is_valid_domain_empty() {
        assert_eq!(Domain::is_valid_domain("".as_bytes(), false), false);
    }

    #[test]
    fn is_valid_domain_length() {
        let a63: String = (0..63).map(|_| "a").collect();
        let a61: String = (0..61).map(|_| "a").collect();
        let mut domain: String = String::new();
        for _ in 0..3 {
            domain.push_str(&a63);
            domain.push_str(".");
        }
        domain.push_str(&a61);

        assert_eq!(
            Domain::is_valid_domain(domain.clone().as_bytes(), false),
            true
        );

        domain.push_str("a");
        assert_eq!(
            Domain::is_valid_domain(domain.clone().as_bytes(), false),
            false
        );
    }
}

#[cfg(test)]
mod conversion_tests {
    use crate::{Authority, Domain, Endpoint, Host};

    #[test]
    fn to_endpoint() {
        assert_eq!(
            Domain::LOCALHOST.to_endpoint(80),
            Endpoint::new(Domain::LOCALHOST, 80)
        );
    }

    #[test]
    fn to_host() {
        assert_eq!(Domain::LOCALHOST.to_host(), Host::Name(Domain::LOCALHOST));
    }

    #[test]
    fn to_authority() {
        assert_eq!(
            Domain::LOCALHOST.to_authority(80),
            Authority::Name(Domain::LOCALHOST.to_endpoint(80))
        );
    }
}

#[cfg(test)]
mod from_tests {
    use std::convert::TryFrom;

    use crate::Domain;

    #[test]
    fn from_u8() {
        assert_eq!(Domain::try_from("a".as_bytes()), Ok(Domain { name: "a" }));
        assert_eq!(Domain::try_from("".as_bytes()), Err(()));
    }

    #[test]
    fn from_str() {
        assert_eq!(Domain::try_from("a"), Ok(Domain { name: "a" }));
        assert_eq!(Domain::try_from(""), Err(()));
    }
}

#[cfg(test)]
mod property_tests {
    use crate::Domain;

    #[test]
    fn name() {
        assert_eq!(Domain::LOCALHOST.name(), "localhost")
    }
}

#[cfg(test)]
mod display_tests {
    use crate::Domain;

    #[test]
    fn display() {
        assert_eq!(Domain::LOCALHOST.to_string(), "localhost")
    }
}
