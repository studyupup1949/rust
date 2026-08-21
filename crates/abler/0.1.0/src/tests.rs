use strum::VariantArray;

use super::*;

#[test]
fn from_u32_strict() {
    assert_eq!(0_u32.in_abler::<true>(), Ok(Abler::Disable));
    assert_eq!(1_u32.in_abler::<true>(), Ok(Abler::Enable));
    assert!(2_u32.in_abler::<true>().is_err());
    assert!(u32::MAX.in_abler::<true>().is_err());
}

#[test]
fn from_u32_relaxed() {
    assert_eq!(0_u32.in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!(1_u32.in_abler::<false>(), Ok(Abler::Enable));
    assert_eq!(2_u32.in_abler::<false>(), Ok(Abler::Enable));
    assert_eq!(u32::MAX.in_abler::<false>(), Ok(Abler::Enable));
}

#[test]
fn from_char_strict() {
    assert_eq!('0'.in_abler::<true>(), Ok(Abler::Disable));
    assert_eq!('1'.in_abler::<true>(), Ok(Abler::Enable));
    assert!('2'.in_abler::<true>().is_err());
    assert!('9'.in_abler::<true>().is_err());
}

#[test]
fn from_char_relaxed() {
    assert_eq!('0'.in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!('1'.in_abler::<false>(), Ok(Abler::Enable));
    assert_eq!('2'.in_abler::<false>(), Ok(Abler::Enable));
    assert_eq!('9'.in_abler::<false>(), Ok(Abler::Enable));
}

#[test]
fn from_str_strict() {
    assert_eq!("0".in_abler::<true>(), Ok(Abler::Disable));
    assert_eq!("1".in_abler::<true>(), Ok(Abler::Enable));
    assert!("2".in_abler::<true>().is_err());
    assert!("123456".in_abler::<true>().is_err());
}

#[test]
fn from_str_relaxed() {
    assert_eq!("0".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("1".in_abler::<false>(), Ok(Abler::Enable));
    assert_eq!("2".in_abler::<false>(), Ok(Abler::Enable));
    assert_eq!("123456".in_abler::<false>(), Ok(Abler::Enable));
}

#[test]
fn from_word_ok_disable() {
    assert_eq!("disable".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("false".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("off".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("no".in_abler::<false>(), Ok(Abler::Disable));

    assert_eq!("DISABLE".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("FALSE".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("OFF".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("NO".in_abler::<false>(), Ok(Abler::Disable));

    assert_eq!("DiSaBlE".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("fAlSe".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("OfF".in_abler::<false>(), Ok(Abler::Disable));
    assert_eq!("nO".in_abler::<false>(), Ok(Abler::Disable));
}

#[test]
fn from_word_ok_enable() {
    assert_eq!("enable".in_abler::<true>(), Ok(Abler::Enable));
    assert_eq!("true".in_abler::<true>(), Ok(Abler::Enable));
    assert_eq!("on".in_abler::<true>(), Ok(Abler::Enable));
    assert_eq!("yes".in_abler::<true>(), Ok(Abler::Enable));

    assert_eq!("ENABLE".in_abler::<true>(), Ok(Abler::Enable));
    assert_eq!("TRUE".in_abler::<true>(), Ok(Abler::Enable));
    assert_eq!("ON".in_abler::<true>(), Ok(Abler::Enable));
    assert_eq!("YES".in_abler::<true>(), Ok(Abler::Enable));

    assert_eq!("EnAbLe".in_abler::<true>(), Ok(Abler::Enable));
    assert_eq!("TrUe".in_abler::<true>(), Ok(Abler::Enable));
    assert_eq!("On".in_abler::<true>(), Ok(Abler::Enable));
    assert_eq!("YeS".in_abler::<true>(), Ok(Abler::Enable));
}

#[test]
fn from_word_fail() {
    assert!("".in_abler::<false>().is_err());
    assert!("not-a-number".in_abler::<false>().is_err());
    assert!("4294967299".in_abler::<false>().is_err());
}

#[test]
fn into_u8() {
    assert_eq!(u8::from(Abler::Disable), 0_u8);
    assert_eq!(u8::from(Abler::Enable), 1_u8);
}

#[test]
fn into_i8() {
    assert_eq!(i8::from(Abler::Disable), 0_i8);
    assert_eq!(i8::from(Abler::Enable), 1_i8);
}

#[test]
fn into_str_default() {
    assert_eq!(<&str>::from(Abler::Disable), "disable");
    assert_eq!(<&str>::from(Abler::Enable), "enable");
}

#[test]
fn into_str_diff_kinds() {
    assert_eq!(<&str>::from(Abler::Disable.display(Kind::Able)), "disable");
    assert_eq!(<&str>::from(Abler::Enable.display(Kind::Able)), "enable");

    assert_eq!(<&str>::from(Abler::Disable.display(Kind::Bool)), "false");
    assert_eq!(<&str>::from(Abler::Enable.display(Kind::Bool)), "true");

    assert_eq!(<&str>::from(Abler::Disable.display(Kind::Switch)), "off");
    assert_eq!(<&str>::from(Abler::Enable.display(Kind::Switch)), "on");

    assert_eq!(<&str>::from(Abler::Disable.display(Kind::Spoken)), "no");
    assert_eq!(<&str>::from(Abler::Enable.display(Kind::Spoken)), "yes");
}

#[test]
fn matching() {
    for &kind in Kind::VARIANTS {
        let names = kind.names();
        assert_eq!(names.len(), 2, "kind '{}' has {} names", kind, names.len());
        assert_eq!(names[0].in_abler::<true>(), Ok(Abler::Disable), "kind '{}' negative name is '{}'", kind, names[0]);
        assert_eq!(names[1].in_abler::<true>(), Ok(Abler::Enable), "kind '{}' positive name is '{}'", kind, names[1]);
    }
}