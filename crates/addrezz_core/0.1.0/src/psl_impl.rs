//! Public suffix / eTLD+1 extraction via the `psl` crate.
//!
//! Gated behind the `psl` feature. Uses the compiled-in Mozilla Public
//! Suffix List to distinguish registered domains from subdomains.

use psl::Psl;

use crate::{Addr, Host};

impl Addr {
    /// Return the public suffix (eTLD) of the host, e.g. `co.uk` for
    /// `api.bbc.co.uk`. Returns `None` for IP hosts.
    pub fn public_suffix(&self) -> Option<&str> {
        if let Host::Domain(d) = &self.host {
            let suffix = psl::List.suffix(d.as_bytes())?;
            core::str::from_utf8(suffix.as_bytes()).ok()
        } else {
            None
        }
    }

    /// Return the registered domain (eTLD+1), e.g. `bbc.co.uk` for
    /// `api.bbc.co.uk`. Returns `None` for IP hosts or when the host
    /// itself is a public suffix.
    pub fn registered_domain(&self) -> Option<&str> {
        if let Host::Domain(d) = &self.host {
            let domain = psl::List.domain(d.as_bytes())?;
            core::str::from_utf8(domain.as_bytes()).ok()
        } else {
            None
        }
    }

    /// Return the subdomain portion, if any. For `api.bbc.co.uk` this
    /// returns `Some("api")`. Returns `None` if there is no subdomain.
    pub fn subdomain(&self) -> Option<&str> {
        if let Host::Domain(d) = &self.host {
            let reg = self.registered_domain()?;
            if d.len() > reg.len() + 1 {
                Some(&d[..d.len() - reg.len() - 1])
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Addr;

    #[test]
    fn public_suffix_co_uk() {
        let a = Addr::parse("https://api.bbc.co.uk/news").unwrap();
        assert_eq!(a.public_suffix(), Some("co.uk"));
    }

    #[test]
    fn registered_domain_co_uk() {
        let a = Addr::parse("https://api.bbc.co.uk/news").unwrap();
        assert_eq!(a.registered_domain(), Some("bbc.co.uk"));
    }

    #[test]
    fn subdomain_extraction() {
        let a = Addr::parse("https://api.bbc.co.uk/").unwrap();
        assert_eq!(a.subdomain(), Some("api"));
    }

    #[test]
    fn no_subdomain_when_at_etld_plus_one() {
        let a = Addr::parse("https://bbc.co.uk/").unwrap();
        assert_eq!(a.subdomain(), None);
    }

    #[test]
    fn simple_tld() {
        let a = Addr::parse("https://docs.rust-lang.org/").unwrap();
        assert_eq!(a.public_suffix(), Some("org"));
        assert_eq!(a.registered_domain(), Some("rust-lang.org"));
        assert_eq!(a.subdomain(), Some("docs"));
    }

    #[test]
    fn ip_returns_none() {
        let a = Addr::parse("http://127.0.0.1/").unwrap();
        assert_eq!(a.public_suffix(), None);
        assert_eq!(a.registered_domain(), None);
    }
}
