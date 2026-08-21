use crate::{Authority, Host, IPAddress, IPv4Address, IPv6Address, SocketAddress};

#[test]
fn to_v4() {
    let result: Option<IPv4Address> = IPv4Address::LOCALHOST.to_ip().to_v4();
    let expected: Option<IPv4Address> = Some(IPv4Address::LOCALHOST);
    assert_eq!(result, expected);

    let result: Option<IPv4Address> = IPv6Address::LOCALHOST.to_ip().to_v4();
    let expected: Option<IPv4Address> = None;
    assert_eq!(result, expected);
}

#[test]
fn to_v6() {
    let result: Option<IPv6Address> = IPv4Address::LOCALHOST.to_ip().to_v6();
    let expected: Option<IPv6Address> = None;
    assert_eq!(result, expected);

    let result: Option<IPv6Address> = IPv6Address::LOCALHOST.to_ip().to_v6();
    let expected: Option<IPv6Address> = Some(IPv6Address::LOCALHOST);
    assert_eq!(result, expected);
}

#[test]
fn to_socket() {
    let result: SocketAddress = IPv4Address::LOCALHOST.to_ip().to_socket(80);
    let expected: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
    assert_eq!(result, expected);
}

#[test]
fn to_host() {
    let result: Host = IPv4Address::LOCALHOST.to_ip().to_host();
    let expected: Host = Host::Address(IPAddress::V4(IPv4Address::LOCALHOST));
    assert_eq!(result, expected);
}

#[test]
fn to_authority() {
    let result: Authority = IPv4Address::LOCALHOST.to_ip().to_authority(80);
    let expected: Authority =
        Authority::new(Host::Address(IPAddress::V4(IPv4Address::LOCALHOST)), 80);
    assert_eq!(result, expected);
}

#[test]
fn from_u8_4() {
    let result: IPAddress = IPAddress::from([0x01, 0x23, 0x45, 0x67]);
    let expected: IPAddress = IPv4Address::new([0x01, 0x23, 0x45, 0x67]).to_ip();
    assert_eq!(result, expected);
}

#[test]
fn from_tuple() {
    let result: IPAddress = IPAddress::from([0x01, 0x23, 0x45, 0x67]);
    let expected: IPAddress = IPv4Address::new([0x01, 0x23, 0x45, 0x67]).to_ip();
    assert_eq!(result, expected);
}

#[test]
fn from_u32() {
    let result: IPAddress = IPAddress::from(0x01234567u32);
    let expected: IPAddress = IPv4Address::new([0x01, 0x23, 0x45, 0x67]).to_ip();
    assert_eq!(result, expected);
}

#[test]
fn from_v4() {
    let result: IPAddress = IPAddress::from(IPv4Address::new([0x01, 0x23, 0x45, 0x67]));
    let expected: IPAddress = IPv4Address::new([0x01, 0x23, 0x45, 0x67]).to_ip();
    assert_eq!(result, expected);
}

#[test]
fn from_u8_16() {
    let result: IPAddress = IPAddress::from([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]);
    let expected: IPAddress = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ])
    .to_ip();
    assert_eq!(result, expected);
}

#[test]
fn from_u16_8() {
    let result: IPAddress = IPAddress::from([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]);
    let expected: IPAddress = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ])
    .to_ip();
    assert_eq!(result, expected);
}

#[test]
fn from_u128() {
    let result: IPAddress = IPAddress::from(0x0123456789ABCDEF0123456789ABCDEFu128);
    let expected: IPAddress = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ])
    .to_ip();
    assert_eq!(result, expected);
}

#[test]
fn from_v6() {
    let result: IPAddress = IPAddress::from(IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]));
    let expected: IPAddress = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ])
    .to_ip();
    assert_eq!(result, expected);
}
