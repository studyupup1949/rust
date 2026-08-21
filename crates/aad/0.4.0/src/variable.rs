use crate::gradients::Gradients;
use crate::operation_record::OperationRecord;
use crate::tape::Tape;

#[derive(Clone, Copy, Debug)]
pub struct Variable<'a> {
    pub(crate) index: usize,
    pub(crate) tape: &'a Tape,
    pub(crate) value: f64,
}

type BinaryFn<T, S = T> = fn(T, S) -> T;
type UnaryFn<T> = fn(T) -> T;
type BinaryPairFn<T> = fn(T, T) -> (T, T);

impl Variable<'_> {
    #[inline]
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }

    #[inline]
    #[must_use]
    pub fn compute_gradients(&self) -> Gradients {
        let operations = &mut self.tape.operations.borrow_mut();
        let count = (*operations).len();
        let mut grads = vec![0.0_f64; count];
        grads[self.index] = 1.0_f64;

        for (i, operation) in (*operations).iter().enumerate().rev() {
            let grad = grads[i];
            if grad == 0.0_f64 {
                continue;
            }
            for j in 0..2 {
                grads[operation.0[j].0] += operation.0[j].1 * grad;
            }
        }

        Gradients(grads)
    }

    #[inline]
    #[must_use]
    pub fn apply_unary_function(self, f: UnaryFn<f64>, df: UnaryFn<f64>) -> Self {
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

    #[inline]
    #[must_use]
    pub fn apply_scalar_function<T: Copy>(
        self,
        f: BinaryFn<f64, T>,
        df: BinaryFn<f64, T>,
        scalar: T,
    ) -> Self {
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

    #[inline]
    #[must_use]
    pub fn apply_binary_function(
        self,
        other: Self,
        f: BinaryFn<f64>,
        dfdx: BinaryPairFn<f64>,
    ) -> Self {
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
