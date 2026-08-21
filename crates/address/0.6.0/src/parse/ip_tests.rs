use std::str::FromStr;

use crate::{IPAddress, IPv4Address, IPv6Address};

#[test]
fn parse_v4() {
    let test_cases: &[(&str, Result<IPv4Address, ()>)] = &[
        ("", Err(())),
        ("127.0.0", Err(())),
        ("127.0.0.1.0", Err(())),
        ("127.0.0.1", Ok(IPv4Address::LOCALHOST)),
        ("-1.0.0.0", Err(())),
        ("0.0.0.-1", Err(())),
        ("256.0.0.0", Err(())),
        ("0.0.0.256", Err(())),
        ("0.0.0.0", Ok(IPv4Address::UNSPECIFIED)),
        ("255.255.255.255", Ok(IPv4Address::BROADCAST)),
        ("[0.0.0.0]", Err(())),
    ];
    for (s, expected) in test_cases {
        let result: Result<IPv4Address, ()> = IPv4Address::from_str(s);
        assert_eq!(result, *expected);
    }
}

#[test]
fn parse_v6() {
    let test_cases: &[(&str, Result<IPv6Address, ()>)] = &[
        ("", Err(())),
        ("::", Ok(IPv6Address::UNSPECIFIED)),
        ("::1", Ok(IPv6Address::LOCALHOST)),
        ("::127.0.0.1", Ok(IPv4Address::LOCALHOST.to_v6_compatible())),
        (
            "::ffff:127.0.0.1",
            Ok(IPv4Address::LOCALHOST.to_v6_mapped()),
        ),
        (
            "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
            Ok(IPv6Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128)),
        ),
        (
            "1:1:1:1:1:1:1:1",
            Ok(IPv6Address::from([1, 1, 1, 1, 1, 1, 1, 1])),
        ),
        (
            "1:1:1:1:1:1:1:1",
            Ok(IPv6Address::from([1, 1, 1, 1, 1, 1, 1, 1])),
        ),
        ("1:1:1:1:1:1:1:1:1", Err(())),
        ("1:1:1:1:1:1:1", Err(())),
        ("1::1", Ok(IPv6Address::from([1, 0, 0, 0, 0, 0, 0, 1]))),
        ("1::1::1", Err(())),
        ("[::]", Err(())),
    ];
    for (s, expected) in test_cases {
        let result: Result<IPv6Address, ()> = IPv6Address::from_str(s);
        assert_eq!(result, *expected);
    }
}

#[test]
fn parse_ip() {
    let test_cases: &[(&str, Result<IPAddress, ()>)] = &[
        ("127.0.0.1", Ok(IPv4Address::LOCALHOST.to_ip())),
        ("::1", Ok(IPv6Address::LOCALHOST.to_ip())),
    ];
    for (s, expected) in test_cases {
        let result: Result<IPAddress, ()> = IPAddress::from_str(s);
        assert_eq!(result, *expected);
    }
}
