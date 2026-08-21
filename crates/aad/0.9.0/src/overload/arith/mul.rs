use std::ops::{Mul, MulAssign};

use crate::{Tape, Variable, operation_record::OperationRecord};

impl<'a, F: Mul<F, Output = F> + Copy> Mul<Self> for &Variable<'a, F> {
    type Output = Variable<'a, F>;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        #[inline]
        fn create_index<F: Mul<F, Output = F> + Copy>(
            value: F,
            rhs: F,
            idx: [usize; 2],
            tape: &Tape<F>,
        ) -> usize {
            let operations = &mut tape.operations.borrow_mut();
            let count = (*operations).len();
            (*operations).push(OperationRecord([(idx[0], rhs), (idx[1], value)]));
            count
        }

        let value = self.value * rhs.value;

        match (self.index, rhs.index) {
            (Some((i, tape)), Some((j, _))) => Variable {
                index: Some((create_index(self.value, rhs.value, [i, j], tape), tape)),
                value,
            },
            (None, None) => Variable { index: None, value },
            (None, Some((j, tape))) => Variable {
                index: Some((
                    create_index(self.value, rhs.value, [usize::MAX, j], tape),
                    tape,
                )),
                value,
            },
            (Some((i, tape)), None) => Variable {
                index: Some((
                    create_index(self.value, rhs.value, [i, usize::MAX], tape),
                    tape,
                )),
                value,
            },
        }
    }
}

impl<'a, F> Mul<Variable<'a, F>> for &Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Mul<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;
    #[inline]
    fn mul(self, rhs: Variable<'a, F>) -> Self::Output {
        self.mul(&rhs)
    }
}

impl<'a, F> Mul<Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Mul<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        (&self).mul(&rhs)
    }
}

impl<'a, F> Mul<&Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Mul<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;

    #[inline]
    fn mul(self, rhs: &Variable<'a, F>) -> Self::Output {
        (&self).mul(rhs)
    }
}

impl<'a, F> MulAssign<Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Mul<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = &*self * &rhs;
    }
}

impl<'a, F> MulAssign<&Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Mul<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    #[inline]
    fn mul_assign(&mut self, rhs: &Self) {
        *self = &*self * rhs;
    }
}
