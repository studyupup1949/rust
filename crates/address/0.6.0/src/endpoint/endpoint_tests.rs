use crate::{Domain, Endpoint};

#[test]
fn new() {
    let endpoint: Endpoint = Endpoint::new(Domain::localhost(), 80);
    assert_eq!(endpoint.domain(), &Domain::localhost());
    assert_eq!(endpoint.port(), 80);
}

#[test]
fn domain() {
    let endpoint: Endpoint = Endpoint::new(Domain::localhost(), 80);
    assert_eq!(endpoint.domain(), &Domain::localhost());
}

#[test]
fn port() {
    let endpoint: Endpoint = Endpoint::new(Domain::localhost(), 80);
    assert_eq!(endpoint.port(), 80);
}

#[test]
fn export() {
    let (domain, port) = Endpoint::new(Domain::localhost(), 80).export();
    assert_eq!(domain, Domain::localhost());
    assert_eq!(port, 80);
}

#[test]
fn export_domain() {
    let domain: Domain = Endpoint::new(Domain::localhost(), 80).export_domain();
    assert_eq!(domain, Domain::localhost());
}
