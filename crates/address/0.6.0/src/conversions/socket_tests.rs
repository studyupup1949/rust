use crate::{
    Authority, Host, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4,
    SocketAddressV6,
};

#[test]
fn v4_to_socket() {
    let result: SocketAddress = IPv4Address::LOCALHOST.to_socket_v4(80).to_socket();
    let expected: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
    assert_eq!(result, expected);
}

#[test]
fn v4_to_authority() {
    let result: Authority = IPv4Address::LOCALHOST.to_socket_v4(80).to_authority();
    let expected: Authority =
        Authority::new(Host::Address(IPAddress::V4(IPv4Address::LOCALHOST)), 80);
    assert_eq!(result, expected);
}

#[test]
fn v4_from_tuple() {
    let result: SocketAddressV4 = (IPv4Address::LOCALHOST, 80).into();
    let expected: SocketAddressV4 = SocketAddressV4::new(IPv4Address::LOCALHOST, 80);
    assert_eq!(result, expected);
}

#[test]
fn v6_to_socket() {
    let result: SocketAddress = IPv6Address::LOCALHOST.to_socket_v6(80).to_socket();
    let expected: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
    assert_eq!(result, expected);
}

#[test]
fn v6_to_authority() {
    let result: Authority = IPv6Address::LOCALHOST.to_socket_v6(80).to_authority();
    let expected: Authority =
        Authority::new(Host::Address(IPAddress::V6(IPv6Address::LOCALHOST)), 80);
    assert_eq!(result, expected);
}

#[test]
fn v6_from_tuple() {
    let result: SocketAddressV6 = (IPv6Address::LOCALHOST, 80).into();
    let expected: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
    assert_eq!(result, expected);
}

#[test]
fn socket_to_v4() {
    let result: Option<SocketAddressV4> = IPv4Address::LOCALHOST.to_socket(80).to_socket_v4();
    let expected: Option<SocketAddressV4> = Some(IPv4Address::LOCALHOST.to_socket_v4(80));
    assert_eq!(result, expected);

    let result: Option<SocketAddressV4> = IPv6Address::LOCALHOST.to_socket(80).to_socket_v4();
    let expected: Option<SocketAddressV4> = None;
    assert_eq!(result, expected);
}

#[test]
fn socket_to_v6() {
    let result: Option<SocketAddressV6> = IPv4Address::LOCALHOST.to_socket(80).to_socket_v6();
    let expected: Option<SocketAddressV6> = None;
    assert_eq!(result, expected);

    let result: Option<SocketAddressV6> = IPv6Address::LOCALHOST.to_socket(80).to_socket_v6();
    let expected: Option<SocketAddressV6> = Some(IPv6Address::LOCALHOST.to_socket_v6(80));
    assert_eq!(result, expected);
}

#[test]
fn socket_to_authority() {
    let result: Authority = IPv4Address::LOCALHOST.to_socket(80).to_authority();
    let expected: Authority = Authority::new(IPv4Address::LOCALHOST.to_host(), 80);
    assert_eq!(result, expected);
}

#[test]
fn socket_from_tuple() {
    let result: SocketAddress = (IPv4Address::LOCALHOST, 80).into();
    let expected: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
    assert_eq!(result, expected);
}

#[test]
fn socket_from_v4() {
    let result: SocketAddress = SocketAddressV4::new(IPv4Address::LOCALHOST, 80).into();
    let expected: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
    assert_eq!(result, expected);
}

#[test]
fn socket_from_v6() {
    let result: SocketAddress = SocketAddressV6::new(IPv6Address::LOCALHOST, 80).into();
    let expected: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
    assert_eq!(result, expected);
}
