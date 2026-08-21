use crate::{Authority, Domain, Host};

#[test]
fn new() {
    let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
    assert_eq!(authority.host(), &Host::Name(Domain::localhost()));
    assert_eq!(authority.port(), 80);
}

#[test]
fn host() {
    let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
    assert_eq!(authority.host(), &Host::Name(Domain::localhost()));
}

#[test]
fn port() {
    let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
    assert_eq!(authority.port(), 80);
}

#[test]
fn export() {
    let (host, port) = Authority::new(Domain::localhost().to_host(), 80).export();
    assert_eq!(host, Domain::localhost().to_host());
    assert_eq!(port, 80);
}

#[test]
fn export_host() {
    let host: Host = Authority::new(Domain::localhost().to_host(), 80).export_host();
    assert_eq!(host, Domain::localhost().to_host());
}
