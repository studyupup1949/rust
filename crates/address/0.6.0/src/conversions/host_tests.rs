use crate::{Authority, Domain, Host, IPAddress, IPv4Address, IPv6Address};

#[test]
fn to_domain() {
    let result: Option<Domain> = Host::Name(Domain::localhost()).to_domain();
    assert_eq!(result, Some(Domain::localhost()));

    let result: Option<Domain> = Host::Address(IPv4Address::LOCALHOST.to_ip()).to_domain();
    assert_eq!(result, None);
}

#[test]
fn to_ip() {
    let result: Option<IPAddress> = Host::Name(Domain::localhost()).to_ip();
    assert_eq!(result, None);

    let result: Option<IPAddress> = Host::Address(IPv4Address::LOCALHOST.to_ip()).to_ip();
    assert_eq!(result, Some(IPv4Address::LOCALHOST.to_ip()));
}

#[test]
fn to_authority() {
    let result: Authority = Domain::localhost().to_authority(80);
    let expected: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
    assert_eq!(result, expected);
}

#[test]
fn from_domain() {
    let result: Host = Domain::localhost().into();
    assert_eq!(result, Host::Name(Domain::localhost()));
}

#[test]
fn from_v4() {
    let result: Host = IPv4Address::LOCALHOST.into();
    assert_eq!(result, Host::Address(IPv4Address::LOCALHOST.to_ip()));
}

#[test]
fn from_v6() {
    let result: Host = IPv6Address::LOCALHOST.into();
    assert_eq!(result, Host::Address(IPv6Address::LOCALHOST.to_ip()));
}

#[test]
fn from_ip() {
    let result: Host = IPv4Address::LOCALHOST.to_ip().into();
    assert_eq!(result, Host::Address(IPv4Address::LOCALHOST.to_ip()));
}
