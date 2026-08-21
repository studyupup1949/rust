//! ADXL355 data types.

/// Raw 20-bit acceleration data (sign-extended to i32).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawXyz {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Acceleration in floating-point units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccelXyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
