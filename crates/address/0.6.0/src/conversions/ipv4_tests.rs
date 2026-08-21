use crate::{Authority, Host, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4};

#[test]
fn to_v6_compatible() {
    let result: IPv6Address = IPv4Address::LOCALHOST.to_v6_compatible();
    let expected: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 0, 0, 1]);
    assert_eq!(result, expected);
}

#[test]
fn to_v6_mapped() {
    let result: IPv6Address = IPv4Address::LOCALHOST.to_v6_mapped();
    let expected: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0x7F, 0, 0, 1]);
    assert_eq!(result, expected);
}

#[test]
fn to_ip() {
    let result: IPAddress = IPv4Address::LOCALHOST.to_ip();
    let expected: IPAddress = IPAddress::V4(IPv4Address::LOCALHOST);
    assert_eq!(result, expected);
}

#[test]
fn to_socket_v4() {
    let result: SocketAddressV4 = IPv4Address::LOCALHOST.to_socket_v4(80);
    let expected: SocketAddressV4 = SocketAddressV4::new(IPv4Address::LOCALHOST, 80);
    assert_eq!(result, expected);
}

#[test]
fn to_socket() {
    let result: SocketAddress = IPv4Address::LOCALHOST.to_socket(80);
    let expected: SocketAddress = SocketAddress::new(IPAddress::V4(IPv4Address::LOCALHOST), 80);
    assert_eq!(result, expected);
}

#[test]
fn to_host() {
    let result: Host = IPv4Address::LOCALHOST.to_host();
    let expected: Host = Host::Address(IPAddress::V4(IPv4Address::LOCALHOST));
    assert_eq!(result, expected);
}

#[test]
fn to_authority() {
    let result: Authority = IPv4Address::LOCALHOST.to_authority(80);
    let expected: Authority =
        Authority::new(Host::Address(IPAddress::V4(IPv4Address::LOCALHOST)), 80);
    assert_eq!(result, expected);
}

#[test]
fn from_u8_4() {
    let result: IPv4Address = IPv4Address::from([0x01, 0x23, 0x45, 0x67]);
    let expected: IPv4Address = IPv4Address::new([0x01, 0x23, 0x45, 0x67]);
    assert_eq!(result, expected);
}

#[test]
fn from_tuple() {
    let result: IPv4Address = IPv4Address::from((0x01, 0x23, 0x45, 0x67));
    let expected: IPv4Address = IPv4Address::new([0x01, 0x23, 0x45, 0x67]);
    assert_eq!(result, expected);
}

#[test]
fn from_u32() {
    let result: IPv4Address = IPv4Address::from(0x01234567);
    let expected: IPv4Address = IPv4Address::new([0x01, 0x23, 0x45, 0x67]);
    assert_eq!(result, expected);
}
