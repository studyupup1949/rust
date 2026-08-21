use num_traits::*;

pub trait Number:
    FromI32 + Float + ConstZero + ConstOne + FloatConst + NumAssignOps + Euclid
{
}
impl Number for f32 {}
impl Number for f64 {}

pub trait FromI32 {
    fn from_i32(n: i32) -> Self;
}
impl FromI32 for f32 {
    fn from_i32(n: i32) -> Self {
        n as Self
    }
}
impl FromI32 for f64 {
    fn from_i32(n: i32) -> Self {
        n as Self
    }
}
