use crate::{IPAddress, IPv4Address, IPv6Address};

#[test]
fn is_v4() {
    assert_eq!(IPAddress::V4(IPv4Address::LOCALHOST).is_v4(), true);
    assert_eq!(IPAddress::V6(IPv6Address::LOCALHOST).is_v4(), false);
}

#[test]
fn is_v6() {
    assert_eq!(IPAddress::V4(IPv4Address::LOCALHOST).is_v6(), false);
    assert_eq!(IPAddress::V6(IPv6Address::LOCALHOST).is_v6(), true);
}
