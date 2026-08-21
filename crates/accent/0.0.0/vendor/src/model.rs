#![allow(unused)]
// model.rs
pub trait Model {
    type Channels<T>;
}

/// ## RGB Model
pub struct RGBmodel;
/// ## HSL/HSV Model
pub struct HSLmodel;
/// ## LAB/LCH Model
pub struct LABmodel;
/// ## CMYK Model
pub struct CMYKmodel;
/// ## Other Common Color Models
pub struct EXTRAmodel;


impl Model for RGBmodel {
    /// [`RGB`]
    type Channels<T> = (T, T, T);
}

impl Model for HSLmodel {
    /// [`HSL`]
    type Channels<T> = (T, T, T);
}

impl Model for LABmodel {
    /// [`LAB`]
    type Channels<T> = (T, T, T);
}

impl Model for CMYKmodel {
    /// [`CMYK`]
    type Channels<T> = (T, T, T);
}

impl Model for EXTRAmodel {
    /// [`EXTRA`]
    type Channels<T> = (T, T, T);
}