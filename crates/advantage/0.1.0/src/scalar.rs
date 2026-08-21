use super::*;
use std::ops::{Add, AddAssign};
use std::ops::{Div, DivAssign};
use std::ops::{Mul, MulAssign};
use std::ops::{Sub, SubAssign};

/// Type supporting all arithmetic operations resulting in a certain type
pub trait Arithmetic<R, T>:
    Sized
    + Clone
    + Copy
    + num::Float
    + Add<R, Output = T>
    + Sub<R, Output = T>
    + Mul<R, Output = T>
    + Div<R, Output = T>
{
}

impl<L, R, T> Arithmetic<R, T> for L where
    L: Sized
        + Clone
        + Copy
        + num::Float
        + Add<R, Output = T>
        + Sub<R, Output = T>
        + Mul<R, Output = T>
        + Div<R, Output = T>
{
}
assert_impl_all!(f64: Arithmetic<f64, f64>);
assert_impl_all!(ADouble: Arithmetic<ADouble, ADouble>);
assert_impl_all!(ADouble: Arithmetic<f64, ADouble>);

/// Type supporting all arithmetic assignments
pub trait ArithmeticAssign<R>:
    Arithmetic<R, Self> + AddAssign<R> + SubAssign<R> + MulAssign<R> + DivAssign<R>
{
}

impl<L, R> ArithmeticAssign<R> for L where
    L: Arithmetic<R, L> + AddAssign<R> + SubAssign<R> + MulAssign<R> + DivAssign<R>
{
}
assert_impl_all!(f64: ArithmeticAssign<f64>);
assert_impl_all!(ADouble: ArithmeticAssign<ADouble>);
assert_impl_all!(ADouble: ArithmeticAssign<f64>);

/// Type supporting all arithmetic operations and assignments
pub trait Scalar<S>:
    From<S>
    + Arithmetic<Self, Self>
    + Arithmetic<S, Self>
    + ArithmeticAssign<Self>
    + ArithmeticAssign<S>
{
}

impl<T, S> Scalar<S> for T where
    T: From<S> + Arithmetic<T, T> + Arithmetic<S, T> + ArithmeticAssign<T> + ArithmeticAssign<S>
{
}
assert_impl_all!(f64: Scalar<f64>);
assert_impl_all!(ADouble: Scalar<f64>);
