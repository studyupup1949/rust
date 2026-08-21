use crate::gradients::Gradients;
use crate::operation_record::OperationRecord;
use crate::tape::Tape;
use num_traits::{Float, Zero};

#[derive(Clone, Copy, Debug)]
pub struct Variable<'a, F> {
    pub(crate) index: usize,
    pub(crate) tape: &'a Tape<F>,
    pub(crate) value: F,
}

type BinaryFn<T, S = T> = fn(T, S) -> T;
type UnaryFn<T> = fn(T) -> T;
type BinaryPairFn<T> = fn(T, T) -> (T, T);

impl<F: Copy> Variable<'_, F> {
    #[inline]
    #[must_use]
    pub const fn value(&self) -> F {
        self.value
    }
    #[inline]
    #[must_use]
    pub fn apply_binary_function(self, other: Self, f: BinaryFn<F>, dfdx: BinaryPairFn<F>) -> Self {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                let df = dfdx(self.value, other.value);
                (*operations).push(OperationRecord([(self.index, df.0), (other.index, df.1)]));
                count
            },
            tape: self.tape,
            value: f(self.value, other.value),
        }
    }
}

impl<F: Copy + Zero> Variable<'_, F> {
    #[inline]
    #[must_use]
    pub fn apply_unary_function(self, f: UnaryFn<F>, df: UnaryFn<F>) -> Self {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (self.index, df(self.value)),
                    (0, F::zero()),
                ]));
                count
            },
            tape: self.tape,
            value: f(self.value),
        }
    }

    #[inline]
    #[must_use]
    pub fn apply_scalar_function<T: Copy>(
        self,
        f: BinaryFn<F, T>,
        df: BinaryFn<F, T>,
        scalar: T,
    ) -> Self {
        Variable {
            index: {
                let operations = &mut self.tape.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (self.index, df(self.value, scalar)),
                    (0, F::zero()),
                ]));
                count
            },
            tape: self.tape,
            value: f(self.value, scalar),
        }
    }
}

impl<F: Float + std::ops::AddAssign> Variable<'_, F> {
    #[inline]
    #[must_use]
    pub fn compute_gradients(&self) -> Gradients<F> {
        let operations = &mut self.tape.operations.borrow_mut();
        let count = (*operations).len();
        let mut grads = vec![F::zero(); count];
        grads[self.index] = F::one();

        for (i, operation) in (*operations).iter().enumerate().rev() {
            let grad = grads[i];
            if grad.is_zero() {
                continue;
            }
            for j in 0..2 {
                grads[operation.0[j].0] += operation.0[j].1 * grad;
            }
        }

        Gradients(grads)
    }
}
