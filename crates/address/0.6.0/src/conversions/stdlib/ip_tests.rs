use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{IPAddress, IPv4Address, IPv6Address};

#[test]
fn v4_to_std() {
    let result: Ipv4Addr = IPv4Address::LOCALHOST.to_std();
    assert_eq!(result, Ipv4Addr::LOCALHOST);
}

#[test]
fn std_from_v4() {
    let result: Ipv4Addr = Ipv4Addr::from(IPv4Address::LOCALHOST);
    assert_eq!(result, Ipv4Addr::LOCALHOST);
}

#[test]
fn v4_from_std() {
    let result: IPv4Address = IPv4Address::from(Ipv4Addr::LOCALHOST);
    assert_eq!(result, IPv4Address::LOCALHOST);
}

#[test]
fn v6_to_std() {
    let result: Ipv6Addr = IPv6Address::LOCALHOST.to_std();
    assert_eq!(result, Ipv6Addr::LOCALHOST);
}

#[test]
fn std_from_v6() {
    let result: Ipv6Addr = Ipv6Addr::from(IPv6Address::LOCALHOST);
    assert_eq!(result, Ipv6Addr::LOCALHOST);
}

#[test]
fn v6_from_std() {
    let result: IPv6Address = IPv6Address::from(Ipv6Addr::LOCALHOST);
    assert_eq!(result, IPv6Address::LOCALHOST);
}

#[test]
fn ip_to_std() {
    let result: IpAddr = IPv4Address::LOCALHOST.to_ip().to_std();
    assert_eq!(result, IpAddr::V4(IPv4Address::LOCALHOST.to_std()));

    let result: IpAddr = IPv6Address::LOCALHOST.to_ip().to_std();
    assert_eq!(result, IpAddr::V6(IPv6Address::LOCALHOST.to_std()));
}

#[test]
fn std_from_ip() {
    let result: IpAddr = IpAddr::from(IPv4Address::LOCALHOST.to_ip());
    assert_eq!(result, IpAddr::V4(Ipv4Addr::LOCALHOST));

    let result: IpAddr = IpAddr::from(IPv6Address::LOCALHOST.to_ip());
    assert_eq!(result, IpAddr::V6(Ipv6Addr::LOCALHOST));
}

#[test]
fn ip_from_std() {
    let result: IPAddress = IPAddress::from(IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(result, IPv4Address::LOCALHOST.to_ip());

    let result: IPAddress = IPAddress::from(IpAddr::V6(Ipv6Addr::LOCALHOST));
    assert_eq!(result, IPv6Address::LOCALHOST.to_ip());
}
