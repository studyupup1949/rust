use crate::ParseError::InvalidAuthority;
use crate::{parse_port, strip_brackets};
use crate::{HostRef, IPv6Address, ParseError};
use std::str::FromStr;

/// A host reference with an associated port.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct AuthorityRef<'a> {
    host: HostRef<'a>,
    port: u16,
}

impl<'a> AuthorityRef<'a> {
    //! Construction

    /// Creates a new authority reference.
    pub const fn new(host: HostRef<'a>, port: u16) -> Self {
        Self { host, port }
    }
}

impl<'a, H: Into<HostRef<'a>>> From<(H, u16)> for AuthorityRef<'a> {
    fn from(tuple: (H, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1)
    }
}

impl<'a> TryFrom<&'a str> for AuthorityRef<'a> {
    type Error = ParseError;

    /// A domain name must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Authority::from_str` to parse mixed-case domain names.
    fn try_from(authority: &'a str) -> Result<Self, Self::Error> {
        let (s, port) = parse_port(authority)?;
        if let Some(s) = strip_brackets(s) {
            let host: HostRef = IPv6Address::from_str(s)?.to_host_ref();
            Ok(host.to_authority_ref(port))
        } else {
            let host: HostRef = HostRef::try_from(s)?;
            if let HostRef::Address(ip) = host {
                if ip.is_v6() {
                    return Err(InvalidAuthority);
                }
            }
            Ok(host.to_authority_ref(port))
        }
    }
}

impl<'a> AuthorityRef<'a> {
    //! Properties

    /// Gets the host reference.
    pub const fn host(self) -> HostRef<'a> {
        self.host
    }

    /// Gets the port.
    pub const fn port(self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use crate::{AuthorityRef, DomainRef, HostRef};

    #[test]
    fn construction() {
        let authority: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        assert_eq!(authority.host, HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port, 80);

        let authority: AuthorityRef = (DomainRef::LOCALHOST, 80).into();
        assert_eq!(authority.host, HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port, 80);
    }

    #[test]
    fn properties() {
        let authority: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        assert_eq!(authority.host(), HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port(), 80);
    }
}
