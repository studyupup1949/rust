use crate::{Addr, ParseError};

// reqwest::Url is a re-export of url::Url but we provide these impls
// independently so consumers on `reqwest` without `url` still get them.

impl TryFrom<Addr> for reqwest::Url {
    type Error = ParseError;
    fn try_from(a: Addr) -> Result<Self, Self::Error> {
        reqwest::Url::parse(&a.to_string()).map_err(|e| ParseError::Invalid(e.to_string()))
    }
}

impl TryFrom<&reqwest::Url> for Addr {
    type Error = ParseError;
    fn try_from(u: &reqwest::Url) -> Result<Self, Self::Error> {
        Addr::parse(u.as_str())
    }
}
