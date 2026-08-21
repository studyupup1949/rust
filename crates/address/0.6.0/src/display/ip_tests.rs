use crate::{IPv4Address, IPv6Address};

#[test]
fn display_v4() {
    let test_cases: &[(IPv4Address, &str)] = &[
        (IPv4Address::UNSPECIFIED, "0.0.0.0"),
        (IPv4Address::LOCALHOST, "127.0.0.1"),
        (IPv4Address::BROADCAST, "255.255.255.255"),
    ];
    for (ip, expected) in test_cases {
        let result: String = ip.to_string();
        assert_eq!(result, *expected);
    }
}

#[test]
fn display_v6() {
    let test_cases: &[(IPv6Address, &str)] = &[
        (IPv6Address::UNSPECIFIED, "::"),
        (IPv6Address::LOCALHOST, "::1"),
        (IPv4Address::LOCALHOST.to_v6_compatible(), "::127.0.0.1"),
        (IPv4Address::LOCALHOST.to_v6_mapped(), "::ffff:127.0.0.1"),
        (
            IPv6Address::from([
                0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
            ]),
            "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
        ),
        (
            IPv6Address::from([
                0xFEDC, 0xBA98, 0x7654, 0x3210, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
            ]),
            "fedc:ba98:7654:3210:ffff:ffff:ffff:ffff",
        ),
        (IPv6Address::from([1, 0, 0, 0, 0, 0, 0, 1]), "1::1"),
        (IPv6Address::from([1, 0, 0, 0, 0, 0, 0, 0]), "1::"),
        (IPv6Address::from([0, 0, 0, 0, 0, 1, 0, 0]), "::1:0:0"),
        (IPv6Address::from([0, 0, 0, 1, 0, 0, 0, 1]), "::1:0:0:0:1"),
        (IPv6Address::from([1, 0, 0, 1, 0, 0, 0, 1]), "1:0:0:1::1"),
        (IPv6Address::from([1, 0, 0, 0, 1, 0, 0, 1]), "1::1:0:0:1"),
        (IPv6Address::from([1, 0, 0, 1, 0, 0, 1, 0]), "1::1:0:0:1:0"),
        (
            IPv6Address::from([1, 0, 1, 0, 1, 0, 1, 0]),
            "1:0:1:0:1:0:1:0",
        ),
    ];
    for (ip, expected) in test_cases {
        let result: String = ip.to_string();
        assert_eq!(result, *expected);
    }
}

#[test]
fn display_ip() {
    let result: String = IPv4Address::LOCALHOST.to_ip().to_string();
    assert_eq!(result, "127.0.0.1");

    let result: String = IPv6Address::LOCALHOST.to_ip().to_string();
    assert_eq!(result, "::1");
}
