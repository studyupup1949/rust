/// A domain reference.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct DomainRef<'a> {
    name: &'a str,
}

impl<'a> DomainRef<'a> {
    //! Special Domains

    /// The localhost domain reference. (localhost)
    pub const LOCALHOST: Self = Self { name: "localhost" };

    /// The example domain reference. (example.com)
    pub const EXAMPLE: Self = Self {
        name: "example.com",
    };
}

impl<'a> DomainRef<'a> {
    //! Construction

    /// Creates a new domain reference. (no validation is done on the name)
    pub const unsafe fn new(name: &'a str) -> Self {
        Self { name }
    }
}

impl<'a> DomainRef<'a> {
    //! Properties

    /// Gets the name.
    pub const fn name(&self) -> &str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use crate::DomainRef;

    #[test]
    fn specials() {
        assert_eq!(DomainRef::LOCALHOST.name, "localhost");
        assert_eq!(DomainRef::EXAMPLE.name, "example.com");
    }

    #[test]
    fn properties() {
        let domain: DomainRef = unsafe { DomainRef::new("localhost") };
        assert_eq!(domain.name(), "localhost");
    }
}
