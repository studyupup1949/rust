use crate::DomainRef;

/// A domain.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Domain {
    name: String,
}

impl Domain {
    //! Special Domains

    /// Creates the localhost domain. (localhost)
    pub fn localhost() -> Self {
        Self {
            name: DomainRef::LOCALHOST.name().to_string(),
        }
    }

    /// Creates the example domain. (example.com)
    pub fn example() -> Self {
        Self {
            name: DomainRef::EXAMPLE.name().to_string(),
        }
    }
}

impl Domain {
    //! Construction

    /// Creates a new domain. (no validation is done on the name)
    pub unsafe fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }
}

impl Domain {
    //! Properties

    /// Gets the name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl From<Domain> for String {
    fn from(domain: Domain) -> Self {
        domain.name
    }
}

#[cfg(test)]
mod tests {
    use crate::Domain;

    #[test]
    fn specials() {
        assert_eq!(Domain::localhost().name, "localhost");
        assert_eq!(Domain::example().name, "example.com");
    }

    #[test]
    fn properties() {
        let domain: Domain = unsafe { Domain::new("localhost") };
        assert_eq!(domain.name(), "localhost");
    }

    #[test]
    fn export() {
        let domain: Domain = unsafe { Domain::new("localhost") };
        let result: String = domain.into();
        assert_eq!(result, "localhost");
    }
}
