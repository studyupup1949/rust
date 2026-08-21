use crate::{Domain, DomainRef};
use std::fmt::{Debug, Display, Formatter};

impl Debug for Domain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Domain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_ref(), f)
    }
}

impl<'a> Debug for DomainRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a> Display for DomainRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad(self.name())
    }
}

#[cfg(test)]
mod tests {
    use crate::Domain;

    #[test]
    fn display() {
        let domain: Domain = Domain::localhost();

        let result: String = domain.to_string();
        let expected: &str = "localhost";
        assert_eq!(result, expected);
    }
}
