#![allow(unused)]

use act2pal::Palette;

macro_rules! act {
    ($($byte:literal)*) => {
        Palette::from_act(&[$($byte),*])
    };
}

macro_rules! pal {
    ($($r:literal $g:literal $b:literal),*$(,)?) => {
        Palette::new(vec![$(($r, $g, $b).into()),*])
    };
}

pub(crate) use {act, pal};
