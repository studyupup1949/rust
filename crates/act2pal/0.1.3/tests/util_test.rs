mod helper;

use act2pal::*;
use helper::*;

#[test]
fn test_new() {
    let pal = Palette::new(vec![(255, 255, 255).into()]);
    assert_eq!(pal, pal!(255 255 255))
}

#[test]
fn test_collect() {
    let pal = Palette::from_iter([(255, 255, 255).into()]);
    assert_eq!(pal, pal!(255 255 255));
}

#[test]
fn test_slice() {
    let pal = pal!(255 255 255);
    assert_eq!(&*pal, [Color::from((255, 255, 255))]);
}
