/// Represents a domain.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Domain {
    name: String,
}

impl Domain {
    //! Special Domains

    /// Creates the localhost domain. (localhost)
    pub fn localhost() -> Self {
        unsafe { Self::new("localhost".to_string()) }
    }

    /// Creates the example domain. (example.com)
    pub fn example() -> Self {
        unsafe { Self::new("example.com".to_string()) }
    }
}

impl Domain {
    //! Constructors

    /// Creates a new domain. (the name must be valid, including case-sensitivity)
    pub(in crate) unsafe fn new(name: String) -> Self {
        Self { name }
    }
}

impl Domain {
    //! Properties

    /// Gets the name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl Domain {
    //! Deconstructors

    /// Exports the name.
    pub fn export_name(self) -> String {
        self.name
    }
}
