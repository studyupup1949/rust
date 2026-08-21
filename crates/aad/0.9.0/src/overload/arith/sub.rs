use std::ops::{Neg, Sub, SubAssign};

use num_traits::One;

use crate::{Tape, Variable, operation_record::OperationRecord};

impl<'a, F: Sub<F, Output = F> + One + Neg<Output = F> + Copy> Sub<Self> for &Variable<'a, F> {
    type Output = Variable<'a, F>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        #[inline]
        fn create_index<F: Sub<F, Output = F> + One + Neg<Output = F>>(
            idx: [usize; 2],
            tape: &Tape<F>,
        ) -> usize {
            let operations = &mut tape.operations.borrow_mut();
            let count = (*operations).len();
            (*operations).push(OperationRecord([
                (idx[0], F::one()),
                (idx[1], F::one().neg()),
            ]));
            count
        }

        let value = self.value - rhs.value;

        match (self.index, rhs.index) {
            (Some((i, tape)), Some((j, _))) => Variable {
                index: Some((create_index([i, j], tape), tape)),
                value,
            },
            (None, None) => Variable { index: None, value },
            (None, Some((j, tape))) => Variable {
                index: Some((create_index([usize::MAX, j], tape), tape)),
                value,
            },
            (Some((i, tape)), None) => Variable {
                index: Some((create_index([i, usize::MAX], tape), tape)),
                value,
            },
        }
    }
}

impl<'a, F> Sub<Variable<'a, F>> for &Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Sub<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;

    #[inline]
    fn sub(self, rhs: Variable<'a, F>) -> Self::Output {
        self.sub(&rhs)
    }
}

impl<'a, F> Sub<Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Sub<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        (&self).sub(&rhs)
    }
}

impl<'a, F> Sub<&Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Sub<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;

    #[inline]
    fn sub(self, rhs: &Self) -> Self::Output {
        (&self).sub(rhs)
    }
}

impl<'a, F> SubAssign<Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Sub<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = &*self - &rhs;
    }
}

impl<'a, F> SubAssign<&Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Sub<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: &Self) {
        *self = &*self - rhs;
    }
}
