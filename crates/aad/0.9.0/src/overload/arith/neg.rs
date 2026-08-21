use std::ops::Neg;

use num_traits::{One, Zero};

use crate::{Variable, operation_record::OperationRecord};

impl<'a, F: Neg<Output = F> + One + Zero + Copy> Neg for &Variable<'a, F> {
    type Output = Variable<'a, F>;
    #[inline]
    fn neg(self) -> Self::Output {
        let value = self.value.neg();
        match self.index {
            Some((i, tape)) => Variable {
                index: {
                    let operations = &mut tape.operations.borrow_mut();
                    let count = (*operations).len();
                    (*operations).push(OperationRecord([
                        (i, F::one().neg()),
                        (usize::MAX, F::zero()),
                    ]));
                    Some((count, tape))
                },
                value,
            },
            None => Variable { index: None, value },
        }
    }
}

impl<'a, F> Neg for Variable<'a, F>
where
    for<'b> &'b Variable<'a, F>: Neg<Output = Variable<'a, F>>,
{
    type Output = Variable<'a, F>;
    #[inline]
    fn neg(self) -> Self::Output {
        (&self).neg()
    }
}
