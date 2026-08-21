use crate::gradients::Gradients;
use crate::operation_record::OperationRecord;
use crate::tape::Tape;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug)]
pub struct Variable<'a> {
    pub(crate) index: usize,
    pub(crate) tape: &'a Tape,
    pub(crate) value: f64,
}

type BinaryFn<T> = fn(T, T) -> T;
type UnaryFn<T> = fn(T) -> T;

impl<'a> Variable<'a> {
    #[inline]
    pub fn value(self) -> f64 {
        self.value
    }

    #[inline]
    pub fn compute_gradients(self) -> Gradients {
        let operations = &mut self.tape.operations.borrow_mut();
        let count = (*operations).len();
        let mut grads = vec![0.0; count];
        grads[self.index] = 1.0;

        for (i, operation) in (*operations).iter().enumerate().rev() {
            let grad = grads[i];
            if grad == 0.0 {
                continue;
            }
            for j in 0..2 {
                grads[operation.0[j].0] += operation.0[j].1 * grad;
            }
        }

        Gradients(grads)
    }

    #[inline]
    pub fn sin(self) -> Variable<'a> {
        self.apply_unary_function(f64::sin, f64::cos)
    }

    #[inline]
    pub fn ln(self) -> Variable<'a> {
        self.apply_unary_function(f64::ln, f64::recip)
    }

    #[inline]
    pub fn powf(self, power: f64) -> Variable<'a> {
        self.apply_scalar_function(f64::powf, |x, power| power * x.powf(power - 1.0), power)
    }

    #[inline]
    pub fn exp(self) -> Variable<'a> {
        self.apply_unary_function(f64::exp, f64::exp)
    }

    #[inline]
    pub fn sqrt(self) -> Variable<'a> {
        self.apply_unary_function(f64::sqrt, |x| 0.5 * x.sqrt().recip())
    }

    #[inline]
    pub fn cos(self) -> Variable<'a> {
        self.apply_unary_function(f64::cos, |x| -f64::sin(x))
    }

    #[inline]
    pub fn tan(self) -> Variable<'a> {
        self.apply_unary_function(f64::tan, |x| f64::cos(x).powi(2).recip())
    }

    #[inline]
    pub fn sinh(self) -> Variable<'a> {
        self.apply_unary_function(f64::sinh, f64::cosh)
    }

    #[inline]
    pub fn cosh(self) -> Variable<'a> {
        self.apply_unary_function(f64::cosh, f64::sinh)
    }

    #[inline]
    pub fn tanh(self) -> Variable<'a> {
        self.apply_unary_function(f64::tanh, |x| f64::cosh(x).powi(2).recip())
    }

    #[inline]
    pub fn recip(self) -> Variable<'a> {
        self.apply_unary_function(f64::recip, |x| -(x * x).recip())
    }

    #[inline(always)]
    pub fn apply_unary_function(&self, f: UnaryFn<f64>, df: UnaryFn<f64>) -> Variable<'a> {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, df(self.value)), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: f(self.value),
        }
    }

    #[inline(always)]
    pub fn apply_scalar_function(
        &self,
        f: BinaryFn<f64>,
        df: BinaryFn<f64>,
        scalar: f64,
    ) -> Variable<'a> {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (self.index, df(self.value, scalar)),
                    (0, 0.0),
                ]));
                count
            },
            tape: self.tape,
            value: f(self.value, scalar),
        }
    }

    #[inline(always)]
    pub fn apply_binary_function(
        &self,
        other: &Variable<'a>,
        f: BinaryFn<f64>,
        dfdx: BinaryFn<f64>,
        dfdy: BinaryFn<f64>,
    ) -> Variable<'a> {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (self.index, dfdx(self.value, other.value)),
                    (other.index, dfdy(self.value, other.value)),
                ]));
                count
            },
            tape: self.tape,
            value: f(self.value, other.value),
        }
    }
}

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

impl<'a> Add<Variable<'a>> for Variable<'a> {
    type Output = Self;

    #[inline]
    fn add(self, other: Variable<'a>) -> Self::Output {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, 1.0), (other.index, 1.0)]));
                count
            },
            tape: self.tape,
            value: self.value + other.value,
        }
    }
}

impl<'a> Sub<Variable<'a>> for Variable<'a> {
    type Output = Self;

    #[inline]
    fn sub(self, other: Variable<'a>) -> Self::Output {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, 1.0), (other.index, -1.0)]));
                count
            },
            tape: self.tape,
            value: self.value - other.value,
        }
    }
}

impl<'a> Mul<Variable<'a>> for Variable<'a> {
    type Output = Self;

    #[inline]
    fn mul(self, other: Variable<'a>) -> Self::Output {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (self.index, other.value),
                    (other.index, self.value),
                ]));
                count
            },
            tape: self.tape,
            value: self.value * other.value,
        }
    }
}

impl<'a> Div<Variable<'a>> for Variable<'a> {
    type Output = Self;

    #[inline]
    fn div(self, other: Variable<'a>) -> Self::Output {
        self.apply_binary_function(&other, |x, y| x / y, |_, y| y.recip(), |x, y| -x / (y * y))
    }
}

impl<'a> Add<f64> for Variable<'a> {
    type Output = Variable<'a>;

    #[inline]
    fn add(self, scalar: f64) -> Variable<'a> {
        Variable {
            index: {
                let mut operations = self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, 1.0), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: scalar + self.value,
        }
    }
}

impl<'a> Add<Variable<'a>> for f64 {
    type Output = Variable<'a>;

    #[inline]
    fn add(self, var: Variable<'a>) -> Variable<'a> {
        var + self
    }
}

impl<'a> Sub<f64> for Variable<'a> {
    type Output = Variable<'a>;

    #[inline]
    fn sub(self, scalar: f64) -> Self::Output {
        Variable {
            index: {
                let mut operations = self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, 1.0), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: self.value - scalar,
        }
    }
}

impl<'a> Sub<Variable<'a>> for f64 {
    type Output = Variable<'a>;

    #[inline]
    fn sub(self, var: Variable<'a>) -> Variable<'a> {
        -var + self
    }
}

impl<'a> Mul<f64> for Variable<'a> {
    type Output = Variable<'a>;

    #[inline]
    fn mul(self, scalar: f64) -> Variable<'a> {
        Variable {
            index: {
                let mut operations = self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, scalar), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: scalar * self.value,
        }
    }
}

impl<'a> Mul<Variable<'a>> for f64 {
    type Output = Variable<'a>;

    #[inline]
    fn mul(self, var: Variable<'a>) -> Variable<'a> {
        var * self
    }
}

impl<'a> Div<f64> for Variable<'a> {
    type Output = Variable<'a>;

    #[inline]
    fn div(self, scalar: f64) -> Variable<'a> {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(self.index, scalar.recip()), (0, 0.0)]));
                count
            },
            tape: self.tape,
            value: self.value / scalar,
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<'a> Div<Variable<'a>> for f64 {
    type Output = Variable<'a>;

    #[inline]
    fn div(self, var: Variable<'a>) -> Variable<'a> {
        var.recip() * self
    }
}

impl AddAssign<f64> for Variable<'_> {
    #[inline]
    fn add_assign(&mut self, scalar: f64) {
        *self = *self + scalar
    }
}

impl<'a> AddAssign<Variable<'a>> for Variable<'a> {
    #[inline]
    fn add_assign(&mut self, other: Variable<'a>) {
        *self = *self + other
    }
}

impl SubAssign<f64> for Variable<'_> {
    #[inline]
    fn sub_assign(&mut self, scalar: f64) {
        *self = *self - scalar
    }
}

impl<'a> SubAssign<Variable<'a>> for Variable<'a> {
    #[inline]
    fn sub_assign(&mut self, other: Variable<'a>) {
        *self = *self - other
    }
}

impl MulAssign<f64> for Variable<'_> {
    #[inline]
    fn mul_assign(&mut self, scalar: f64) {
        *self = *self * scalar
    }
}

impl<'a> MulAssign<Variable<'a>> for Variable<'a> {
    #[inline]
    fn mul_assign(&mut self, other: Variable<'a>) {
        *self = *self * other
    }
}

impl DivAssign<f64> for Variable<'_> {
    #[inline]
    fn div_assign(&mut self, scalar: f64) {
        *self = *self / scalar
    }
}

impl<'a> DivAssign<Variable<'a>> for Variable<'a> {
    #[inline]
    fn div_assign(&mut self, other: Variable<'a>) {
        *self = *self / other
    }
}
