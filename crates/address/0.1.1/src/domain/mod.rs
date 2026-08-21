use std::convert::TryFrom;

/// Represents a domain name.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Domain {
    name: String
}

impl Domain {

    /// The maximum length of a domain label.
    pub const MAX_LABEL_LENGTH: usize = 63;

    /// The maximum length of a domain name.
    pub const MAX_DOMAIN_LENGTH: usize = 253;

    /// Checks if the domain label is valid. This function is case insensitive.
    ///
    /// ```
    /// use address::domain::Domain;
    ///
    /// assert!(!Domain::is_valid_label("".as_bytes()));
    /// assert!(Domain::is_valid_label("a".as_bytes()));
    /// assert!(Domain::is_valid_label("A".as_bytes()));
    /// assert!(Domain::is_valid_label("0".as_bytes()));
    /// assert!(!Domain::is_valid_label("a-".as_bytes()));
    /// assert!(!Domain::is_valid_label("-a".as_bytes()));
    /// assert!(Domain::is_valid_label("a-b".as_bytes()));
    /// ```
    pub fn is_valid_label(label: &[u8]) -> bool {
        if (1..Domain::MAX_LABEL_LENGTH).contains(&label.len()) {
            for (i, r) in label.iter().enumerate() {
                let c: u8 = *r;
                if !c.is_ascii_alphanumeric() {
                    if c == b'-' {
                        if i == 0 || i == label.len() - 1 {
                            return false;
                        } else if label[i-1] == b'-' || label[i+1] == b'-' {
                            return false;
                        }
                    }
                }
            }
            return true;
        }
        return false;
    }

    /// Checks if the domain name is valid. This function is case-insensitive.
    ///
    /// ```
    /// use address::domain::Domain;
    ///
    /// assert!(!Domain::is_valid_domain("".as_bytes()));
    /// assert!(Domain::is_valid_domain("a".as_bytes()));
    /// assert!(!Domain::is_valid_domain("a.".as_bytes()));
    /// assert!(!Domain::is_valid_domain(".a".as_bytes()));
    /// assert!(Domain::is_valid_domain("a.b".as_bytes()));
    /// ```
    pub fn is_valid_domain(name: &[u8]) -> bool {
        if (1..Domain::MAX_DOMAIN_LENGTH).contains(&name.len()) {
            let mut rem: &[u8] = name;
            loop {
                match rem.iter().position(|r| *r == b'.') {
                    None => return Domain::is_valid_label(&rem),
                    Some(i) => {
                        if !Domain::is_valid_label(&rem[..i]) {
                            return false;
                        }
                        rem = &rem[i+1..];
                    }
                }
            }
        }
        return false;
    }
}

impl ToString for Domain {

    /// ```
    /// use address::domain::Domain;
    /// use std::convert::TryFrom;
    ///
    /// assert_eq!(Domain::try_from("a").ok().unwrap().to_string(), "a");
    /// ```
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl TryFrom<&[u8]> for Domain {
    type Error = &'static str;

    /// ```
    /// use address::domain::Domain;
    /// use std::convert::TryFrom;
    ///
    /// let e: Result<Domain, &'static str> = Err("Invalid Domain");
    /// assert_eq!(Domain::try_from("".as_bytes()), e);
    /// assert_eq!(Domain::try_from("a".as_bytes()).ok().unwrap().to_string(), "a");
    /// assert_eq!(Domain::try_from("a.".as_bytes()), e);
    /// assert_eq!(Domain::try_from(".a".as_bytes()), e);
    /// assert_eq!(Domain::try_from("a.b".as_bytes()).ok().unwrap().to_string(), "a.b");
    /// ```
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if Domain::is_valid_domain(value) {
            let lower: Vec<u8> = value.to_ascii_lowercase();
            let name: String = unsafe { String::from_utf8_unchecked(lower) };
            Ok(Domain { name })
        } else {
            Err("Invalid Domain")
        }
    }
}

impl TryFrom<&str> for Domain {
    type Error = &'static str;

    /// ```
    /// use address::domain::Domain;
    /// use std::convert::TryFrom;
    ///
    /// let e: Result<Domain, &'static str> = Err("Invalid Domain");
    /// assert_eq!(Domain::try_from(""), e);
    /// assert_eq!(Domain::try_from("a").ok().unwrap().to_string(), "a");
    /// assert_eq!(Domain::try_from("a."), e);
    /// assert_eq!(Domain::try_from(".a"), e);
    /// assert_eq!(Domain::try_from("a.b").ok().unwrap().to_string(), "a.b");
    /// ```
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Domain::try_from(value.as_bytes())
    }
}
