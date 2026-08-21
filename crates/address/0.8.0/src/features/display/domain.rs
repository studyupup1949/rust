use std::fmt::{Display, Formatter};

use crate::{Domain, DomainRef};

impl Display for Domain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ref())
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
        let domain: Domain = Domain::localhost();
        assert_eq!(domain.to_string(), "localhost");
    }

    #[test]
    fn domain_ref() {
        let domain: Domain = Domain::localhost();
        let domain: DomainRef = domain.to_ref();
        assert_eq!(domain.to_string(), "localhost");
    }
}
