use std::ops::{Add, AddAssign};

use num_traits::One;

use crate::{Tape, Variable, operation_record::OperationRecord};

impl<'a, F: Add<F, Output = F> + One + Copy> Add<Self> for &Variable<'a, F> {
    type Output = Variable<'a, F>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        #[inline]
        fn create_index<F: Add<F, Output = F> + One>(idx: [usize; 2], tape: &Tape<F>) -> usize {
            let operations = &mut tape.operations.borrow_mut();
            let count = (*operations).len();
            (*operations).push(OperationRecord([(idx[0], F::one()), (idx[1], F::one())]));
            count
        }

        let value = self.value + rhs.value;

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

impl<'a, F> Add<Variable<'a, F>> for &Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Add<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;

    #[inline]
    fn add(self, rhs: Variable<'a, F>) -> Self::Output {
        self.add(&rhs)
    }
}

impl<'a, F> Add<Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Add<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        (&self).add(&rhs)
    }
}

impl<'a, F> Add<&Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Add<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;

    #[inline]
    fn add(self, rhs: &Variable<'a, F>) -> Self::Output {
        (&self).add(rhs)
    }
}

impl<'a, F> AddAssign<Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Add<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = &*self + &rhs;
    }
}

impl<'a, F> AddAssign<&Self> for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Add<&'b Variable<'a, F>, Output = Variable<'a, F>>,
{
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        *self = &*self + rhs;
    }
}
