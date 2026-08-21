use crate::IPv4Address;

#[test]
fn specials() {
    assert_eq!(IPv4Address::UNSPECIFIED.address(), [0, 0, 0, 0]);
    assert_eq!(IPv4Address::LOCALHOST.address(), [127, 0, 0, 1]);
    assert_eq!(IPv4Address::BROADCAST.address(), [255, 255, 255, 255]);
}

#[test]
fn new() {
    let result: IPv4Address = IPv4Address::new([0x01, 0x23, 0x45, 0x67]);
    assert_eq!(result.address(), [0x01, 0x23, 0x45, 0x67]);
}

#[test]
fn default() {
    let result: IPv4Address = IPv4Address::default();
    let expected: IPv4Address = IPv4Address::UNSPECIFIED;
    assert_eq!(result, expected);
}

#[test]
fn address() {
    let result: [u8; 4] = IPv4Address::new([0x01, 0x23, 0x45, 0x67]).address();
    let expected: [u8; 4] = [0x01, 0x23, 0x45, 0x67];
    assert_eq!(result, expected);
}

#[test]
fn bytes() {
    let result: (u8, u8, u8, u8) = IPv4Address::new([0x01, 0x23, 0x45, 0x67]).bytes();
    let expected: (u8, u8, u8, u8) = (0x01, 0x23, 0x45, 0x67);
    assert_eq!(result, expected);
}
