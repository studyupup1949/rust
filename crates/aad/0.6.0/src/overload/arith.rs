use crate::operation_record::OperationRecord;
use crate::variable::Variable;
use num_traits::{Float, Inv, One, Zero};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl<F: Neg<Output = F> + One + Zero> Neg for Variable<'_, F> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Variable {
            index: {
                let mut operations = self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (self.index, F::one().neg()),
                    (0, F::zero()),
                ]));
                count
            },
            tape: self.tape,
            value: -self.value,
        }
    }
}

impl<F: Add<F, Output = F> + One> Add<Self> for Variable<'_, F> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (self.index, F::one()),
                    (rhs.index, F::one()),
                ]));
                count
            },
            tape: self.tape,
            value: self.value + rhs.value,
        }
    }
}

impl<F: Sub<F, Output = F> + One + Neg<Output = F>> Sub<Self> for Variable<'_, F> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (self.index, F::one()),
                    (rhs.index, -F::one()),
                ]));
                count
            },
            tape: self.tape,
            value: self.value - rhs.value,
        }
    }
}

impl<F: Mul<F, Output = F> + Copy> Mul<Self> for Variable<'_, F> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (self.index, rhs.value),
                    (rhs.index, self.value),
                ]));
                count
            },
            tape: self.tape,
            value: self.value * rhs.value,
        }
    }
}

impl<F: Copy + Div<F, Output = F> + Inv<Output = F> + Neg<Output = F> + Mul<Output = F>> Div<Self>
    for Variable<'_, F>
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.apply_binary_function(rhs, |x, y| x / y, |x, y| (y.inv(), -x / (y * y)))
    }
}

impl<F: Copy + One + Zero> AddAssign<Self> for Variable<'_, F> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<F: Copy + One + Neg<Output = F> + Sub<F, Output = F>> SubAssign<Self> for Variable<'_, F> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<F: Copy + Mul<F, Output = F>> MulAssign<Self> for Variable<'_, F> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<F: Copy + Div<Self, Output = Self> + Float + Inv<Output = F>> DivAssign<Self>
    for Variable<'_, F>
{
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}
