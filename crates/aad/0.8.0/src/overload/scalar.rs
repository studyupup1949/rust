use crate::operation_record::OperationRecord;
use crate::Variable;
use num_traits::{Inv, NumCast, One, Zero};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

macro_rules! impl_scalar_add {
    ($scalar:ty) => {
        impl<F: Add<$scalar, Output = F> + One + Zero> Add<$scalar> for Variable<'_, F> {
            type Output = Self;

            #[inline]
            fn add(self, rhs: $scalar) -> Self::Output {
                let value = self.value + rhs;
                match self.index {
                    Some((i, tape)) => Variable {
                        index: {
                            let mut operations = tape.operations.borrow_mut();
                            let count = operations.len();
                            operations
                                .push(OperationRecord([(i, F::one()), (usize::MAX, F::zero())]));
                            Some((count, tape))
                        },
                        value,
                    },
                    None => Variable { index: None, value },
                }
            }
        }

        impl<'a, F: Add<$scalar, Output = F> + One + Zero> Add<Variable<'a, F>> for $scalar {
            type Output = Variable<'a, F>;

            #[inline]
            fn add(self, rhs: Self::Output) -> Self::Output {
                rhs + self
            }
        }

        impl<F: Copy + Add<$scalar, Output = F> + One + Zero> AddAssign<$scalar>
            for Variable<'_, F>
        {
            #[inline]
            fn add_assign(&mut self, rhs: $scalar) {
                *self = *self + rhs;
            }
        }
    };
}

macro_rules! impl_scalar_sub {
    ($scalar:ty) => {
        impl<F: Sub<$scalar, Output = F> + One + Zero> Sub<$scalar> for Variable<'_, F> {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: $scalar) -> Self::Output {
                let value = self.value - rhs;
                match self.index {
                    Some((i, tape)) => Variable {
                        index: {
                            let mut operations = tape.operations.borrow_mut();
                            let count = operations.len();
                            operations
                                .push(OperationRecord([(i, F::one()), (usize::MAX, F::zero())]));
                            Some((count, tape))
                        },
                        value,
                    },
                    None => Variable { index: None, value },
                }
            }
        }

        impl<'a, F: Neg<Output = F> + One + Zero + Sub<$scalar, Output = F>> Sub<Variable<'a, F>>
            for $scalar
        {
            type Output = Variable<'a, F>;

            #[inline]
            fn sub(self, rhs: Self::Output) -> Self::Output {
                -(rhs - self)
            }
        }

        impl<F: Copy + Sub<$scalar, Output = F> + One + Zero> SubAssign<$scalar>
            for Variable<'_, F>
        {
            #[inline]
            fn sub_assign(&mut self, rhs: $scalar) {
                *self = *self - rhs;
            }
        }
    };
}

macro_rules! impl_scalar_mul {
    ($scalar:ty) => {
        impl<F: Copy + Mul<$scalar, Output = F> + Zero + NumCast> Mul<$scalar> for Variable<'_, F> {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: $scalar) -> Self::Output {
                let value = self.value * rhs;
                match self.index {
                    Some((i, tape)) => Variable {
                        index: {
                            let mut operations = tape.operations.borrow_mut();
                            let count = operations.len();
                            operations.push(OperationRecord([
                                (i, F::from(rhs).unwrap()),
                                (usize::MAX, F::zero()),
                            ]));
                            Some((count, tape))
                        },
                        value,
                    },
                    None => Variable { index: None, value },
                }
            }
        }

        impl<'a, F: Copy + Mul<$scalar, Output = F> + Zero + NumCast> Mul<Variable<'a, F>>
            for $scalar
        {
            type Output = Variable<'a, F>;

            #[inline]
            fn mul(self, rhs: Self::Output) -> Self::Output {
                rhs * self
            }
        }

        impl<F: Copy + Mul<$scalar, Output = F> + Zero + NumCast> MulAssign<$scalar>
            for Variable<'_, F>
        {
            #[inline]
            fn mul_assign(&mut self, rhs: $scalar) {
                *self = *self * rhs;
            }
        }
    };
}

macro_rules! impl_scalar_div {
    ($scalar:ty) => {
        impl<F: Div<$scalar, Output = F> + Zero + NumCast> Div<$scalar> for Variable<'_, F> {
            type Output = Self;

            #[inline]
            fn div(self, rhs: $scalar) -> Self::Output {
                let value = self.value / rhs;
                match self.index {
                    Some((i, tape)) => Variable {
                        index: {
                            let operations = &mut tape.operations.borrow_mut();
                            let count = operations.len();
                            operations.push(OperationRecord([
                                (i, F::from(rhs.recip()).unwrap()),
                                (usize::MAX, F::zero()),
                            ]));
                            Some((count, tape))
                        },
                        value,
                    },
                    None => Variable { index: None, value },
                }
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl<
                'a,
                F: Copy
                    + Inv<Output = F>
                    + Zero
                    + Mul<F, Output = F>
                    + Neg<Output = F>
                    + Mul<$scalar, Output = F>
                    + NumCast,
            > Div<Variable<'a, F>> for $scalar
        {
            type Output = Variable<'a, F>;

            #[inline]
            fn div(self, rhs: Self::Output) -> Self::Output {
                rhs.inv() * self
            }
        }

        impl<F: Copy + Div<$scalar, Output = F> + Zero + NumCast> DivAssign<$scalar>
            for Variable<'_, F>
        {
            #[inline]
            fn div_assign(&mut self, rhs: $scalar) {
                *self = *self / rhs;
            }
        }
    };
}

impl_scalar_add!(f32);
impl_scalar_sub!(f32);
impl_scalar_mul!(f32);
impl_scalar_div!(f32);

impl_scalar_add!(f64);
impl_scalar_sub!(f64);
impl_scalar_mul!(f64);
impl_scalar_div!(f64);

impl_scalar_add!(i8);
impl_scalar_sub!(i8);
impl_scalar_mul!(i8);

impl_scalar_add!(i16);
impl_scalar_sub!(i16);
impl_scalar_mul!(i16);

impl_scalar_add!(i32);
impl_scalar_sub!(i32);
impl_scalar_mul!(i32);

impl_scalar_add!(i64);
impl_scalar_sub!(i64);
impl_scalar_mul!(i64);

impl_scalar_add!(i128);
impl_scalar_sub!(i128);
impl_scalar_mul!(i128);

impl_scalar_add!(isize);
impl_scalar_sub!(isize);
impl_scalar_mul!(isize);

impl_scalar_add!(u8);
impl_scalar_sub!(u8);
impl_scalar_mul!(u8);

impl_scalar_add!(u16);
impl_scalar_sub!(u16);
impl_scalar_mul!(u16);

impl_scalar_add!(u32);
impl_scalar_sub!(u32);
impl_scalar_mul!(u32);

impl_scalar_add!(u64);
impl_scalar_sub!(u64);
impl_scalar_mul!(u64);

impl_scalar_add!(u128);
impl_scalar_sub!(u128);
impl_scalar_mul!(u128);

impl_scalar_add!(usize);
impl_scalar_sub!(usize);
impl_scalar_mul!(usize);
