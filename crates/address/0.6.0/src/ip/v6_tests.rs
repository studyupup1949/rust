use crate::IPv6Address;

#[test]
fn specials() {
    let result: [u16; 8] = IPv6Address::UNSPECIFIED.segments();
    let expected: [u16; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(result, expected);

    let result: [u16; 8] = IPv6Address::LOCALHOST.segments();
    let expected: [u16; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
    assert_eq!(result, expected);
}

#[test]
fn new() {
    let ip: IPv6Address = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]);
    let result: &[u8; 16] = ip.address();
    let expected: &[u8; 16] = &[
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ];
    assert_eq!(result, expected);
}

#[test]
fn address() {
    let ip: IPv6Address = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]);
    let result: &[u8; 16] = ip.address();
    let expected: &[u8; 16] = &[
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ];
    assert_eq!(result, expected);
}

#[test]
fn segments() {
    let ip: IPv6Address = IPv6Address::new([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
        0xEF,
    ]);
    let result: [u16; 8] = ip.segments();
    let expected: [u16; 8] = [
        0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
    ];
    assert_eq!(result, expected);
}

#[test]
fn is_v4_compatible() {
    let address: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_compatible(), true);
    let address: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_compatible(), false);
    let address: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_compatible(), false);
    let address: IPv6Address =
        IPv6Address::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_compatible(), false);
}

#[test]
fn is_v4_mapped() {
    let address: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_mapped(), false);
    let address: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_mapped(), true);
    let address: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_mapped(), false);
    let address: IPv6Address =
        IPv6Address::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_mapped(), false);
}

#[test]
fn is_v4_convertable() {
    let address: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_convertable(), true);
    let address: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_convertable(), true);
    let address: IPv6Address =
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_convertable(), false);
    let address: IPv6Address =
        IPv6Address::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 0, 0, 1]);
    assert_eq!(address.is_v4_convertable(), false);
}
