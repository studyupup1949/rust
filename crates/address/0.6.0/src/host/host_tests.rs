use crate::{Domain, Host, IPv4Address};

#[test]
fn is_name() {
    let host: Host = Host::Name(Domain::localhost());
    assert_eq!(host.is_name(), true);

    let host: Host = Host::Address(IPv4Address::LOCALHOST.to_ip());
    assert_eq!(host.is_name(), false);
}

#[test]
fn is_address() {
    let host: Host = Host::Name(Domain::localhost());
    assert_eq!(host.is_address(), false);

    let host: Host = Host::Address(IPv4Address::LOCALHOST.to_ip());
    assert_eq!(host.is_address(), true);
}
