#![allow(unused)]

use act2pal::Colors;

macro_rules! act {
    ($($byte:literal)*) => {
        Colors::from_act(&[$($byte),*])
    };
}

macro_rules! colors {
    ($($r:literal $g:literal $b:literal),*$(,)?) => {
        Colors::new(vec![$(($r, $g, $b).into()),*])
    };
}

pub(crate) use act;
pub(crate) use colors;
