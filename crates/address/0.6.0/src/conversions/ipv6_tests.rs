use crate::{Authority, Host, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV6};

#[test]
fn to_v4() {
    let result: Option<IPv4Address> =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 0, 0, 1]).to_v4();
    let expected: Option<IPv4Address> = Some(IPv4Address::LOCALHOST);
    assert_eq!(result, expected);

    let result: Option<IPv4Address> =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0x7F, 0, 0, 1]).to_v4();
    let expected: Option<IPv4Address> = Some(IPv4Address::LOCALHOST);
    assert_eq!(result, expected);

    let result: Option<IPv4Address> =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x7F, 0, 0, 1]).to_v4();
    let expected: Option<IPv4Address> = None;
    assert_eq!(result, expected);

    let result: Option<IPv4Address> =
        IPv6Address::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 0, 0, 1]).to_v4();
    let expected: Option<IPv4Address> = None;
    assert_eq!(result, expected);
}

#[test]
fn to_ip() {
    let result: IPAddress = IPv6Address::LOCALHOST.to_ip();
    let expected: IPAddress = IPAddress::V6(IPv6Address::LOCALHOST);
    assert_eq!(result, expected);
}

#[test]
fn to_socket_v6() {
    let result: SocketAddressV6 = IPv6Address::LOCALHOST.to_socket_v6(80);
    let expected: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
    assert_eq!(result, expected);
}

#[test]
fn to_socket() {
    let result: SocketAddress = IPv6Address::LOCALHOST.to_socket(80);
    let expected: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
    assert_eq!(result, expected);
}

#[test]
fn to_host() {
    let result: Host = IPv6Address::LOCALHOST.to_host();
    let expected: Host = Host::Address(IPAddress::V6(IPv6Address::LOCALHOST));
    assert_eq!(result, expected);
}

#[test]
fn to_authority() {
    let result: Authority = IPv6Address::LOCALHOST.to_authority(80);
    let expected: Authority =
        Authority::new(Host::Address(IPAddress::V6(IPv6Address::LOCALHOST)), 80);
    assert_eq!(result, expected);
}

#[test]
fn from_u8_16() {
    let result: IPv6Address = IPv6Address::from([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]);
    let expected: IPv6Address = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]);
    assert_eq!(result, expected);
}

#[test]
fn from_u16_8() {
    let result: IPv6Address = IPv6Address::from([
        0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
    ]);
    let expected: IPv6Address = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]);
    assert_eq!(result, expected);
}

#[test]
fn from_u128() {
    let result: IPv6Address = IPv6Address::from(0x0123456789ABCDEF0123456789ABCDEFu128);
    let expected: IPv6Address = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]);
    assert_eq!(result, expected);
}
