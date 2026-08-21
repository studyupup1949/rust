use crate::grads::Grads;
use crate::operations::Operation;
use crate::tape::Tape;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug)]
pub struct Var<'a> {
    pub(crate) idx: usize,
    pub(crate) tape: &'a Tape,
    pub(crate) value: f64,
}

unsafe impl Send for Var<'_> {}
unsafe impl Sync for Var<'_> {}

type BinaryFn<T> = fn(T, T) -> T;
type UnaryFn<T> = fn(T) -> T;

impl<'a> Var<'a> {
    #[inline]
    pub fn value(&self) -> f64 {
        self.value
    }

    #[inline]
    pub fn backward(&self) -> Grads {
        unsafe {
            let operations = self.tape.operations.get();
            let count = (*operations).len();
            let mut grads = vec![0.0; count];
            *grads.get_unchecked_mut(self.idx) = 1.0;

            for (i, operation) in (*operations).iter().enumerate().rev() {
                let grad = *grads.get_unchecked(i);
                if grad == 0.0 {
                    continue;
                }
                for j in 0..2 {
                    *grads.get_unchecked_mut(operation.0.get_unchecked(j).0) +=
                        operation.0.get_unchecked(j).1 * grad;
                }
            }

            Grads(grads)
        }
    }

    #[inline]
    pub fn sin(&self) -> Var<'a> {
        self.custom_unary_fn(f64::sin, f64::cos)
    }

    #[inline]
    pub fn ln(&self) -> Var<'a> {
        self.custom_unary_fn(f64::ln, f64::recip)
    }

    #[inline]
    pub fn powf(&self, power: f64) -> Var<'a> {
        self.custom_scalar_fn(f64::powf, |x, power| power * x.powf(power - 1.0), power)
    }

    #[inline]
    pub fn exp(&self) -> Var<'a> {
        self.custom_unary_fn(f64::exp, f64::exp)
    }

    #[inline]
    pub fn sqrt(&self) -> Var<'a> {
        self.custom_unary_fn(f64::sqrt, |x| 0.5 * x.sqrt().recip())
    }

    #[inline]
    pub fn cos(&self) -> Var<'a> {
        self.custom_unary_fn(f64::cos, |x| -f64::sin(x))
    }

    #[inline]
    pub fn tan(&self) -> Var<'a> {
        self.custom_unary_fn(f64::tan, |x| f64::cos(x).powi(2).recip())
    }

    #[inline]
    pub fn sinh(&self) -> Var<'a> {
        self.custom_unary_fn(f64::sinh, f64::cosh)
    }

    #[inline]
    pub fn cosh(&self) -> Var<'a> {
        self.custom_unary_fn(f64::cosh, f64::sinh)
    }

    #[inline]
    pub fn tanh(&self) -> Var<'a> {
        self.custom_unary_fn(f64::tanh, |x| f64::cosh(x).powi(2).recip())
    }

    #[inline]
    pub fn recip(&self) -> Var<'a> {
        self.custom_unary_fn(f64::recip, |x| -(x * x).recip())
    }

    #[inline(always)]
    pub fn custom_unary_fn(&self, f: UnaryFn<f64>, df: UnaryFn<f64>) -> Var<'a> {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(self.idx, df(self.value)), (0, 0.0)]));
                    count
                },
                tape: self.tape,
                value: f(self.value),
            }
        }
    }

    #[inline(always)]
    pub fn custom_scalar_fn(&self, f: BinaryFn<f64>, df: BinaryFn<f64>, scalar: f64) -> Var<'a> {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(self.idx, df(self.value, scalar)), (0, 0.0)]));
                    count
                },
                tape: self.tape,
                value: f(self.value, scalar),
            }
        }
    }

    #[inline(always)]
    pub fn custom_binary_fn(
        &self,
        other: &Var<'a>,
        f: BinaryFn<f64>,
        dfdx: BinaryFn<f64>,
        dfdy: BinaryFn<f64>,
    ) -> Var<'a> {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([
                        (self.idx, dfdx(self.value, other.value)),
                        (other.idx, dfdy(self.value, other.value)),
                    ]));
                    count
                },
                tape: self.tape,
                value: f(self.value, other.value),
            }
        }
    }
}

impl Neg for Var<'_> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(self.idx, -1.0), (0, 0.0)]));
                    count
                },
                tape: self.tape,
                value: -self.value,
            }
        }
    }
}

impl<'a> Add<Var<'a>> for Var<'a> {
    type Output = Self;

    #[inline]
    fn add(self, other: Var<'a>) -> Self::Output {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(self.idx, 1.0), (other.idx, 1.0)]));
                    count
                },
                tape: self.tape,
                value: self.value + other.value,
            }
        }
    }
}

impl<'a> Sub<Var<'a>> for Var<'a> {
    type Output = Self;

    #[inline]
    fn sub(self, other: Var<'a>) -> Self::Output {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(self.idx, 1.0), (other.idx, -1.0)]));
                    count
                },
                tape: self.tape,
                value: self.value - other.value,
            }
        }
    }
}

impl<'a> Mul<Var<'a>> for Var<'a> {
    type Output = Self;

    #[inline]
    fn mul(self, other: Var<'a>) -> Self::Output {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([
                        (self.idx, other.value),
                        (other.idx, self.value),
                    ]));
                    count
                },
                tape: self.tape,
                value: self.value * other.value,
            }
        }
    }
}

impl<'a> Div<Var<'a>> for Var<'a> {
    type Output = Self;

    #[inline]
    fn div(self, other: Var<'a>) -> Self::Output {
        self.custom_binary_fn(&other, |x, y| x / y, |_, y| y.recip(), |x, y| -x / (y * y))
    }
}

impl<'a> Add<f64> for Var<'a> {
    type Output = Var<'a>;

    #[inline]
    fn add(self, scalar: f64) -> Var<'a> {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(self.idx, 1.0), (0, 0.0)]));
                    count
                },
                tape: self.tape,
                value: scalar + self.value,
            }
        }
    }
}

impl<'a> Add<Var<'a>> for f64 {
    type Output = Var<'a>;

    #[inline]
    fn add(self, var: Var<'a>) -> Var<'a> {
        var + self
    }
}

impl<'a> Sub<f64> for Var<'a> {
    type Output = Var<'a>;

    #[inline]
    fn sub(self, scalar: f64) -> Self::Output {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(self.idx, 1.0), (0, 0.0)]));
                    count
                },
                tape: self.tape,
                value: self.value - scalar,
            }
        }
    }
}

impl<'a> Sub<Var<'a>> for f64 {
    type Output = Var<'a>;

    #[inline]
    fn sub(self, var: Var<'a>) -> Var<'a> {
        -var + self
    }
}

impl<'a> Mul<f64> for Var<'a> {
    type Output = Var<'a>;

    #[inline]
    fn mul(self, scalar: f64) -> Var<'a> {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(self.idx, scalar), (0, 0.0)]));
                    count
                },
                tape: self.tape,
                value: scalar * self.value,
            }
        }
    }
}

impl<'a> Mul<Var<'a>> for f64 {
    type Output = Var<'a>;

    #[inline]
    fn mul(self, var: Var<'a>) -> Var<'a> {
        var * self
    }
}

impl<'a> Div<f64> for Var<'a> {
    type Output = Var<'a>;

    #[inline]
    fn div(self, scalar: f64) -> Var<'a> {
        unsafe {
            Var {
                idx: {
                    let operations = self.tape.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(self.idx, scalar.recip()), (0, 0.0)]));
                    count
                },
                tape: self.tape,
                value: self.value / scalar,
            }
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<'a> Div<Var<'a>> for f64 {
    type Output = Var<'a>;

    #[inline]
    fn div(self, var: Var<'a>) -> Var<'a> {
        var.recip() * self
    }
}

impl AddAssign<f64> for Var<'_> {
    #[inline]
    fn add_assign(&mut self, scalar: f64) {
        *self = *self + scalar
    }
}

impl<'a> AddAssign<Var<'a>> for Var<'a> {
    #[inline]
    fn add_assign(&mut self, other: Var<'a>) {
        *self = *self + other
    }
}

impl SubAssign<f64> for Var<'_> {
    #[inline]
    fn sub_assign(&mut self, scalar: f64) {
        *self = *self - scalar
    }
}

impl<'a> SubAssign<Var<'a>> for Var<'a> {
    #[inline]
    fn sub_assign(&mut self, other: Var<'a>) {
        *self = *self - other
    }
}

impl MulAssign<f64> for Var<'_> {
    #[inline]
    fn mul_assign(&mut self, scalar: f64) {
        *self = *self * scalar
    }
}

impl<'a> MulAssign<Var<'a>> for Var<'a> {
    #[inline]
    fn mul_assign(&mut self, other: Var<'a>) {
        *self = *self * other
    }
}

impl DivAssign<f64> for Var<'_> {
    #[inline]
    fn div_assign(&mut self, scalar: f64) {
        *self = *self / scalar
    }
}

impl<'a> DivAssign<Var<'a>> for Var<'a> {
    #[inline]
    fn div_assign(&mut self, other: Var<'a>) {
        *self = *self / other
    }
}
