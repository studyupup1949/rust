use std::io::Cursor;

use crate::{
    types::{VarInt, VarLong, Position},
    deserialise::Deserialise,
};

#[test]
fn deserialise_var_int() {
    let mut buf = Cursor::new(vec![0u8]);
    assert_eq!(VarInt::deserialise(&mut buf, ()).unwrap().0, 0);

    let mut buf = Cursor::new(vec![1u8]);
    assert_eq!(VarInt::deserialise(&mut buf, ()).unwrap().0, 1);

    let mut buf = Cursor::new(vec![2u8]);
    assert_eq!(VarInt::deserialise(&mut buf, ()).unwrap().0, 2);

    let mut buf = Cursor::new(vec![127u8]);
    assert_eq!(VarInt::deserialise(&mut buf, ()).unwrap().0, 127);

    let mut buf = Cursor::new(vec![128u8, 1]);
    assert_eq!(VarInt::deserialise(&mut buf, ()).unwrap().0, 128);

    let mut buf = Cursor::new(vec![255u8, 1]);
    assert_eq!(VarInt::deserialise(&mut buf, ()).unwrap().0, 255);

    let mut buf = Cursor::new(vec![255u8, 255, 255, 255, 7]);
    assert_eq!(VarInt::deserialise(&mut buf, ()).unwrap().0, 2_147_483_647);

    let mut buf = Cursor::new(vec![255u8, 255, 255, 255, 15]);
    assert_eq!(VarInt::deserialise(&mut buf, ()).unwrap().0, -1);

    let mut buf = Cursor::new(vec![128u8, 128, 128, 128, 8]);
    assert_eq!(VarInt::deserialise(&mut buf, ()).unwrap().0, -2_147_483_648);
}

#[test]
fn deserialise_var_long() {
    let mut buf = Cursor::new(vec![0u8]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, 0);

    let mut buf = Cursor::new(vec![1u8]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, 1);

    let mut buf = Cursor::new(vec![2u8]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, 2);

    let mut buf = Cursor::new(vec![127u8]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, 127);

    let mut buf = Cursor::new(vec![128u8, 1]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, 128);

    let mut buf = Cursor::new(vec![255u8, 1]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, 255);

    let mut buf = Cursor::new(vec![255u8, 255, 255, 255, 7]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, 0x7FFF_FFFF);

    let mut buf = Cursor::new(vec![255u8, 255, 255, 255, 255, 255, 255, 255, 127]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, 0x7FFF_FFFF_FFFF_FFFF);

    let mut buf = Cursor::new(vec![255u8, 255, 255, 255, 255, 255, 255, 255, 255, 1]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, -1);

    let mut buf = Cursor::new(vec![128u8, 128, 128, 128, 248, 255, 255, 255, 255, 1]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, -0x7FFF_FFFF);

    let mut buf = Cursor::new(vec![128u8, 128, 128, 128, 128, 128, 128, 128, 128, 1]);
    assert_eq!(VarLong::deserialise(&mut buf, ()).unwrap().0, -0x7FFF_FFFF_FFFF_FFFF);
}

#[test]
pub fn deserialise_position() {
    let mut buf = Cursor::new(vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(Position::deserialise(&mut buf, ()).unwrap(), Position(-1, -1, -1));

    let mut buf = Cursor::new(vec![0x80, 0x00, 0x00, 0x60, 0x06, 0x00, 0x00, 0x01]);
    assert_eq!(Position::deserialise(&mut buf, ()).unwrap(), Position(-33_554_431, -2047, -33_554_431));
}
