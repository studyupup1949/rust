use crate::operation_record::OperationRecord;
use crate::Variable;
use num_traits::{Float, NumCast, One, Zero};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

macro_rules! impl_scalar_operations {
    ($scalar:ty) => {
        impl<F: Add<$scalar, Output = F> + One + Zero> Add<$scalar> for Variable<'_, F> {
            type Output = Self;

            #[inline]
            fn add(self, rhs: $scalar) -> Self::Output {
                Variable {
                    index: {
                        let mut operations = self.tape.operations.borrow_mut();
                        let count = operations.len();
                        operations.push(OperationRecord([(self.index, F::one()), (0, F::zero())]));
                        count
                    },
                    tape: self.tape,
                    value: self.value + rhs,
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

        impl<F: Sub<$scalar, Output = F> + One + Zero> Sub<$scalar> for Variable<'_, F> {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: $scalar) -> Self::Output {
                Variable {
                    index: {
                        let mut operations = self.tape.operations.borrow_mut();
                        let count = operations.len();
                        operations.push(OperationRecord([(self.index, F::one()), (0, F::zero())]));
                        count
                    },
                    tape: self.tape,
                    value: self.value - rhs,
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

        impl<F: Copy + Mul<$scalar, Output = F> + Zero + NumCast> Mul<$scalar> for Variable<'_, F> {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: $scalar) -> Self::Output {
                Variable {
                    index: {
                        let mut operations = self.tape.operations.borrow_mut();
                        let count = operations.len();
                        operations.push(OperationRecord([
                            (self.index, F::from(rhs).unwrap()),
                            (0, F::zero()),
                        ]));
                        count
                    },
                    tape: self.tape,
                    value: self.value * rhs,
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

        impl<F: Div<$scalar, Output = F> + Zero + NumCast> Div<$scalar> for Variable<'_, F> {
            type Output = Self;

            #[inline]
            fn div(self, rhs: $scalar) -> Self::Output {
                Variable {
                    index: {
                        let operations = &mut self.tape.operations.borrow_mut();
                        let count = operations.len();
                        operations.push(OperationRecord([
                            (self.index, F::from(rhs.recip()).unwrap()),
                            (0, F::zero()),
                        ]));
                        count
                    },
                    tape: self.tape,
                    value: self.value / rhs,
                }
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl<'a, F: Copy + Mul<$scalar, Output = F> + Float> Div<Variable<'a, F>> for $scalar {
            type Output = Variable<'a, F>;

            #[inline]
            fn div(self, rhs: Self::Output) -> Self::Output {
                rhs.recip() * self
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

        impl<F: Copy + Sub<$scalar, Output = F> + One + Zero> SubAssign<$scalar>
            for Variable<'_, F>
        {
            #[inline]
            fn sub_assign(&mut self, rhs: $scalar) {
                *self = *self - rhs;
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

impl_scalar_operations!(f32);
impl_scalar_operations!(f64);
