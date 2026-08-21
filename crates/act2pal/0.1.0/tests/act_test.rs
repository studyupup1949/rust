mod helper;

use act2pal::*;
use helper::*;

#[test]
fn test_0_colors() {
    assert_eq!(act!(0 0 0).unwrap(), colors!());
}

#[test]
fn test_1_color() {
    assert_eq!(act!(255 255 255 1 0 0).unwrap(), colors!(255 255 255));
}

#[test]
fn test_example() {
    assert_eq!(
        Colors::from_act(include_bytes!("./helper/example.act")).unwrap(),
        colors!(
            112 200 160,
            0 0 0,
            248 248 248,
            107 90 115,
            224 8 8,
            248 184 112,
            32 152 8,
            144 240 144,
            48 80 200,
            160 192 240,
            0 0 0,
            248 248 248,
            0 0 0,
            0 248 152,
            0 200 184,
            72 112 160,
        ),
    );
}
