use crate::Domain;

/// A domain name reference.
///
/// See `Domain` for the domain name validation rules. A borrowed name cannot be normalized, so
/// the name must already be lowercase.
#[must_use]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct DomainRef<'a> {
    name: &'a str,
}

impl<'a> DomainRef<'a> {
    //! Special Domains

    /// The `localhost` domain reference.
    pub const LOCALHOST: Self = Self { name: "localhost" };

    /// The `example.com` domain reference.
    pub const EXAMPLE: Self = Self {
        name: "example.com",
    };
}

impl<'a> DomainRef<'a> {
    //! Construction

    /// Creates a new domain reference.
    ///
    /// # Safety
    /// The `name` must be valid and lowercase. Validity is a struct invariant that unsafe code
    /// may rely on.
    pub unsafe fn new_unchecked(name: &'a str) -> Self {
        debug_assert!(Domain::is_valid_name_str(name, false));

        Self { name }
    }
}

impl<'a> From<&'a Domain> for DomainRef<'a> {
    fn from(domain: &'a Domain) -> Self {
        domain.to_ref()
    }
}

impl<'a> PartialEq<Domain> for DomainRef<'a> {
    fn eq(&self, other: &Domain) -> bool {
        *self == other.to_ref()
    }
}

impl<'a> DomainRef<'a> {
    //! Properties

    /// Gets the name.
    #[must_use]
    pub const fn name(self) -> &'a str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef};

    #[test]
    fn specials() {
        assert_eq!(DomainRef::LOCALHOST.name, "localhost");
        assert_eq!(DomainRef::EXAMPLE.name, "example.com");
    }

    #[test]
    fn construction() {
        let owned: Domain = Domain::localhost();
        let domain: DomainRef = (&owned).into();
        assert_eq!(domain.name, "localhost");
    }

    #[test]
    fn equality() {
        let owned: Domain = Domain::localhost();
        assert_eq!(DomainRef::LOCALHOST, owned);
        assert_ne!(DomainRef::EXAMPLE, owned);
    }

    #[test]
    fn properties() {
        let domain: DomainRef = DomainRef::LOCALHOST;
        assert_eq!(domain.name(), "localhost");
    }
}
