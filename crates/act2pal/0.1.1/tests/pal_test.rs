mod helper;

use act2pal::*;
use helper::*;

#[test]
fn test_0_colors() {
    assert_eq!(colors!().to_pal_string().unwrap(), "JASC-PAL\n0100\n0\n");
}

#[test]
fn test_1_color() {
    assert_eq!(
        colors!(1 2 3).to_pal_string().unwrap(),
        "JASC-PAL\n0100\n1\n1 2 3\n",
    );
}

#[test]
fn test_example() {
    assert_eq!(
        Colors::from_act(include_bytes!("./helper/example.act"))
            .unwrap()
            .to_pal_string()
            .unwrap(),
        "JASC-PAL\n0100\n16\n112 200 160\n0 0 0\n248 248 248\n107 90 115\n224 8 8\n248 184 112\n32 152 8\n144 240 144\n48 80 200\n160 192 240\n0 0 0\n248 248 248\n0 0 0\n0 248 152\n0 200 184\n72 112 160\n"
    )
}
