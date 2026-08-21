#![allow(unused)]
// space.rs
use crate::model::*;
use crate::family::*;


pub trait Space {
    type ModelType: Model;
}

// RGB Family
impl Space for rgb::DP3 {
    type ModelType = RGBmodel;
}
impl Space for rgb::SRGB {
    type ModelType = RGBmodel;
}
impl Space for rgb::LINEAR {
    type ModelType = RGBmodel;
}

// HSL Family
impl Space for hsl::HSL {
    type ModelType = HSLmodel;
}
impl Space for hsl::HSV {
    type ModelType = HSLmodel;
}

// LAB Family
impl Space for lab::LAB {
    type ModelType = LABmodel;
}
impl Space for lab::OKLAB {
    type ModelType = LABmodel;
}
impl Space for lab::OKLCH {
    type ModelType = LABmodel;
}

// EXTRA Family
impl Space for extra::HEX {
    type ModelType = EXTRAmodel;
}
impl Space for extra::XYZ {
    type ModelType = EXTRAmodel;
}