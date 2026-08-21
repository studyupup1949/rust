use crate::operation_record::OperationRecord;
use crate::variable::Variable;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl Neg for Variable<'_> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Variable {
            index: {
                let mut operations = self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, -1.0), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: -self.value,
        }
    }
}

impl Add<Self> for Variable<'_> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, 1.0), (rhs.index, 1.0)]));
                count
            },
            tape: self.tape,
            value: self.value + rhs.value,
        }
    }
}

impl Sub<Self> for Variable<'_> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, 1.0), (rhs.index, -1.0)]));
                count
            },
            tape: self.tape,
            value: self.value - rhs.value,
        }
    }
}

impl Mul<Self> for Variable<'_> {
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

impl Div<Self> for Variable<'_> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.apply_binary_function(rhs, |x, y| x / y, |x, y| (y.recip(), -x / (y * y)))
    }
}

impl Add<f64> for Variable<'_> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: f64) -> Self::Output {
        Variable {
            index: {
                let mut operations = self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, 1.0), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: rhs + self.value,
        }
    }
}

impl<'a> Add<Variable<'a>> for f64 {
    type Output = Variable<'a>;

    #[inline]
    fn add(self, rhs: Self::Output) -> Self::Output {
        rhs + self
    }
}

impl Sub<f64> for Variable<'_> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: f64) -> Self::Output {
        Variable {
            index: {
                let mut operations = self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, 1.0), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: self.value - rhs,
        }
    }
}

impl<'a> Sub<Variable<'a>> for f64 {
    type Output = Variable<'a>;

    #[inline]
    fn sub(self, rhs: Self::Output) -> Self::Output {
        -rhs + self
    }
}

impl Mul<f64> for Variable<'_> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Variable {
            index: {
                let mut operations = self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, rhs), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: rhs * self.value,
        }
    }
}

impl<'a> Mul<Variable<'a>> for f64 {
    type Output = Variable<'a>;

    #[inline]
    fn mul(self, rhs: Self::Output) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Variable<'_> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, rhs.recip()), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: self.value / rhs,
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<'a> Div<Variable<'a>> for f64 {
    type Output = Variable<'a>;

    #[inline]
    fn div(self, rhs: Self::Output) -> Self::Output {
        rhs.recip() * self
    }
}

impl AddAssign<f64> for Variable<'_> {
    #[inline]
    fn add_assign(&mut self, rhs: f64) {
        *self = *self + rhs;
    }
}

impl AddAssign<Self> for Variable<'_> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign<f64> for Variable<'_> {
    #[inline]
    fn sub_assign(&mut self, rhs: f64) {
        *self = *self - rhs;
    }
}

impl SubAssign<Self> for Variable<'_> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f64> for Variable<'_> {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl MulAssign<Self> for Variable<'_> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl DivAssign<f64> for Variable<'_> {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

impl DivAssign<Self> for Variable<'_> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}
