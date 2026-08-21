use crate::{Authority, Domain, Endpoint, IPv4Address, IPv6Address, SocketAddress};

#[test]
fn to_endpoint() {
    let result: Option<Endpoint> = Authority::new(Domain::localhost().to_host(), 80).to_endpoint();
    let expected: Option<Endpoint> = Some(Domain::localhost().to_endpoint(80));
    assert_eq!(result, expected);

    let result: Option<Endpoint> =
        Authority::new(IPv4Address::LOCALHOST.to_host(), 80).to_endpoint();
    let expected: Option<Endpoint> = None;
    assert_eq!(result, expected);
}

#[test]
fn to_socket() {
    let result: Option<SocketAddress> =
        Authority::new(Domain::localhost().to_host(), 80).to_socket();
    let expected: Option<SocketAddress> = None;
    assert_eq!(result, expected);

    let result: Option<SocketAddress> =
        Authority::new(IPv4Address::LOCALHOST.to_host(), 80).to_socket();
    let expected: Option<SocketAddress> = Some(IPv4Address::LOCALHOST.to_socket(80));
    assert_eq!(result, expected);
}

#[test]
fn from_endpoint() {
    let result: Authority = Authority::from(Domain::localhost().to_endpoint(80));
    let expected: Authority = Authority::new(Domain::localhost().to_host(), 80);
    assert_eq!(result, expected);
}

#[test]
fn from_socket_v4() {
    let result: Authority = Authority::from(IPv4Address::LOCALHOST.to_socket_v4(80));
    let expected: Authority = Authority::new(IPv4Address::LOCALHOST.to_host(), 80);
    assert_eq!(result, expected);
}

#[test]
fn from_socket_v6() {
    let result: Authority = Authority::from(IPv6Address::LOCALHOST.to_socket_v6(80));
    let expected: Authority = Authority::new(IPv6Address::LOCALHOST.to_host(), 80);
    assert_eq!(result, expected);
}

#[test]
fn from_socket() {
    let result: Authority = Authority::from(IPv6Address::LOCALHOST.to_socket(80));
    let expected: Authority = Authority::new(IPv6Address::LOCALHOST.to_host(), 80);
    assert_eq!(result, expected);
}

#[test]
fn from_tuple() {
    let result: Authority = (Domain::localhost(), 80).into();
    let expected: Authority = Authority::new(Domain::localhost().to_host(), 80);
    assert_eq!(result, expected);
}

#[test]
fn tuple_from_authority() {
    let (host, port) = Authority::new(Domain::localhost().to_host(), 80).into();
    assert_eq!(host, Domain::localhost().to_host());
    assert_eq!(port, 80);
}
