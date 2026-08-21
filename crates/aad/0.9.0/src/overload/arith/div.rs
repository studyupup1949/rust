use std::ops::{Div, DivAssign, Mul, Neg};

use num_traits::Inv;

use crate::Variable;

impl<'a, F: Copy + Div<F, Output = F> + Inv<Output = F> + Neg<Output = F> + Mul<Output = F>>
    Div<Self> for &Variable<'a, F>
{
    type Output = Variable<'a, F>;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.apply_binary_function(rhs, |x, y| x / y, |x, y| (y.inv(), -x / (y * y)))
    }
}

impl<'a, F> Div<Variable<'a, F>> for &Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Div<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;

    #[inline]
    fn div(self, rhs: Variable<'a, F>) -> Self::Output {
        self.div(&rhs)
    }
}

impl<'a, F> Div<&Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Div<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;

    #[inline]
    fn div(self, rhs: &Self) -> Self::Output {
        (&self).div(rhs)
    }
}

impl<'a, F> Div<Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Div<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        (&self).div(&rhs)
    }
}

impl<'a, F> DivAssign<Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Div<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = &*self / &rhs;
    }
}

impl<'a, F> DivAssign<&Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Div<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    #[inline]
    fn div_assign(&mut self, rhs: &Self) {
        *self = &*self / rhs;
    }
}
