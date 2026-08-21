use crate::variable::Variable;
use num_traits::Inv;
use std::ops::{Mul, Neg};

impl<'a> Inv for Variable<'a, f32> {
    type Output = Variable<'a, f32>;

    #[inline]
    fn inv(self) -> Self::Output {
        self.apply_unary_function(f32::inv, |x| x.mul(x).inv().neg())
    }
}

impl Inv for Variable<'_, f64> {
    type Output = Self;

    #[inline]
    fn inv(self) -> Self::Output {
        self.apply_unary_function(f64::inv, |x| x.mul(x).inv().neg())
    }
}

impl Inv for Variable<'_, Variable<'_, f32>> {
    type Output = Self;

    #[inline]
    fn inv(self) -> Self::Output {
        self.apply_unary_function(Variable::inv, |x| x.mul(x).inv().neg())
    }
}

impl Inv for Variable<'_, Variable<'_, f64>> {
    type Output = Self;

    #[inline]
    fn inv(self) -> Self::Output {
        self.apply_unary_function(Variable::inv, |x| x.mul(x).inv().neg())
    }
}
