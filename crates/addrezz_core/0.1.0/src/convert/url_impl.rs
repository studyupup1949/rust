use crate::{Addr, ParseError};

impl TryFrom<Addr> for url::Url {
    type Error = ParseError;
    fn try_from(a: Addr) -> Result<Self, Self::Error> {
        url::Url::parse(&a.to_string()).map_err(|e| ParseError::Invalid(e.to_string()))
    }
}

impl TryFrom<&url::Url> for Addr {
    type Error = ParseError;
    fn try_from(u: &url::Url) -> Result<Self, Self::Error> {
        Addr::parse(u.as_str())
    }
}

impl TryFrom<url::Url> for Addr {
    type Error = ParseError;
    fn try_from(u: url::Url) -> Result<Self, Self::Error> {
        (&u).try_into()
    }
}
