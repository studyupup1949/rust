use std::fmt::{Display, Formatter};

use crate::{Domain, DomainRef};

impl Display for Domain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> Display for DomainRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef};

    #[test]
    fn domain() {
        assert_eq!(Domain::localhost().to_string(), "localhost");
    }

    #[test]
    fn domain_ref() {
        assert_eq!(DomainRef::LOCALHOST.to_string(), "localhost");
    }
}
