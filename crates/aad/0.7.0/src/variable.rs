use crate::gradients::Gradients;
use crate::operation_record::OperationRecord;
use crate::tape::Tape;
use num_traits::{One, Zero};

#[derive(Clone, Copy, Debug)]
/// A variable type that tracks operations for automatic differentiation.
///
/// This struct represents a variable in the computation graph, storing its value
/// and maintaining references to the tape that records operations performed on it.
///
/// # Type Parameters
///
/// * `'a` - The lifetime of the reference to the tape
/// * `F` - The underlying numeric type (typically `f32` or `f64`)
///
/// # Fields
///
/// * `index` - The unique index of this variable in the computation tape
/// * `tape` - Reference to the tape that records operations on this variable
/// * `value` - The current value of the variable
pub struct Variable<'a, F> {
    pub(crate) index: usize,
    pub(crate) tape: Option<&'a Tape<F>>,
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
    pub fn apply_binary_function(self, rhs: Self, f: BinaryFn<F>, dfdx: BinaryPairFn<F>) -> Self {
        let tape = self.tape.or(rhs.tape);
        match tape {
            Some(tape) => Variable {
                index: {
                    let operations = &mut tape.operations.borrow_mut();
                    let count = (*operations).len();
                    let df = dfdx(self.value, rhs.value);
                    (*operations).push(OperationRecord([(self.index, df.0), (rhs.index, df.1)]));
                    count
                },
                tape: Some(tape),
                value: f(self.value, rhs.value),
            },
            None => Variable {
                index: 0,
                tape: None,
                value: f(self.value, rhs.value),
            },
        }
    }
}

impl<F: Copy + Zero> Variable<'_, F> {
    #[inline]
    #[must_use]
    pub fn apply_unary_function(self, f: UnaryFn<F>, df: UnaryFn<F>) -> Self {
        match self.tape {
            Some(tape) => Variable {
                index: {
                    let operations = &mut tape.operations.borrow_mut();
                    let count = (*operations).len();
                    (*operations).push(OperationRecord([
                        (self.index, df(self.value)),
                        (0, F::zero()),
                    ]));
                    count
                },
                tape: Some(tape),
                value: f(self.value),
            },
            None => Variable {
                index: 0,
                tape: None,
                value: f(self.value),
            },
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
        match self.tape {
            Some(tape) => Variable {
                index: {
                    let operations = &mut tape.operations.borrow_mut();
                    let count = (*operations).len();
                    (*operations).push(OperationRecord([
                        (self.index, df(self.value, scalar)),
                        (0, F::zero()),
                    ]));
                    count
                },
                tape: Some(tape),
                value: f(self.value, scalar),
            },
            None => Variable {
                index: 0,
                tape: None,
                value: f(self.value, scalar),
            },
        }
    }
}

impl<F: Copy + One + Zero> Variable<'_, F> {
    #[inline]
    #[must_use]
    pub fn compute_gradients(&self) -> Gradients<F> {
        let operations = &mut self.tape.unwrap().operations.borrow_mut();
        let count = (*operations).len();
        let mut grads = vec![F::zero(); count];
        grads[self.index] = F::one();

        for (i, operation) in (*operations).iter().enumerate().rev() {
            let grad = grads[i];
            if grad.is_zero() {
                continue;
            }
            for j in 0..2 {
                grads[operation.0[j].0] = grads[operation.0[j].0] + operation.0[j].1 * grad;
            }
        }

        Gradients(grads)
    }
}
