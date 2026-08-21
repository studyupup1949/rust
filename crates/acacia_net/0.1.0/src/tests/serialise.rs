use std::io::Cursor;

use crate::{
    types::{VarInt, VarLong, Position},
    serialise::Serialise,
};

#[test]
fn serialise_var_int() {
    let mut buf = Cursor::new(vec![]);
    VarInt(0).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [0u8]);

    let mut buf = Cursor::new(vec![]);
    VarInt(1).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [1u8]);

    let mut buf = Cursor::new(vec![]);
    VarInt(2).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [2u8]);

    let mut buf = Cursor::new(vec![]);
    VarInt(127).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [127u8]);

    let mut buf = Cursor::new(vec![]);
    VarInt(128).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [128u8, 1]);

    let mut buf = Cursor::new(vec![]);
    VarInt(255).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [255u8, 1]);

    let mut buf = Cursor::new(vec![]);
    VarInt(2_147_483_647).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [255u8, 255, 255, 255, 7]);

    let mut buf = Cursor::new(vec![]);
    VarInt(-1).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [255u8, 255, 255, 255, 15]);

    let mut buf = Cursor::new(vec![]);
    VarInt(-2_147_483_648).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [128u8, 128, 128, 128, 8]);
}

#[test]
fn serialise_var_long() {
    let mut buf = Cursor::new(vec![]);
    VarLong(0).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [0u8]);

    let mut buf = Cursor::new(vec![]);
    VarLong(1).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [1u8]);

    let mut buf = Cursor::new(vec![]);
    VarLong(2).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [2u8]);

    let mut buf = Cursor::new(vec![]);
    VarLong(127).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [127u8]);

    let mut buf = Cursor::new(vec![]);
    VarLong(128).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [128u8, 1]);

    let mut buf = Cursor::new(vec![]);
    VarLong(255).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [255u8, 1]);

    let mut buf = Cursor::new(vec![]);
    VarLong(2_147_483_647).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [255u8, 255, 255, 255, 7]);

    let mut buf = Cursor::new(vec![]);
    VarLong(9_223_372_036_854_775_807).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [255u8, 255, 255, 255, 255, 255, 255, 255, 127]);

    let mut buf = Cursor::new(vec![]);
    VarLong(-1).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [255u8, 255, 255, 255, 255, 255, 255, 255, 255, 1]);

    let mut buf = Cursor::new(vec![]);
    VarLong(-2_147_483_648).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [128u8, 128, 128, 128, 248, 255, 255, 255, 255, 1]);

    let mut buf = Cursor::new(vec![]);
    VarLong(-9_223_372_036_854_775_808).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [128u8, 128, 128, 128, 128, 128, 128, 128, 128, 1]);
}

#[test]
fn serialise_position() {
    let mut buf = Cursor::new(vec![]);
    Position(-1, -1, -1).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

    let mut buf = Cursor::new(vec![]);
    Position(-33_554_431, -2047, -33_554_431).serialise(&mut buf).unwrap();
    assert_eq!(*buf.get_ref(), [0x80, 0x00, 0x00, 0x60, 0x06, 0x00, 0x00, 0x01]);
}
