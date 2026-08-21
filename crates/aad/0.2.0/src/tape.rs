use crate::operations::Operation;
use crate::var::Var;
use std::cell::UnsafeCell;

#[derive(Default)]
pub struct Tape {
    operations: UnsafeCell<Vec<Operation>>,
    pub(crate) values: UnsafeCell<Vec<f64>>,
}

impl Tape {
    pub fn var(&self, value: f64) -> Var {
        unsafe {
            let values = &mut *self.values.get();
            let idx = (*values).len();
            (*values).push(value);
            Var { idx, tape: self }
        }
    }

    #[inline(always)]
    pub(crate) fn record(&self, operation: Operation) {
        unsafe {
            (*self.operations.get()).push(operation);
        }
    }

    pub(crate) fn replay(&self, grads: &mut [f64]) {
        unsafe {
            for operation in (*self.operations.get()).iter().rev() {
                operation.backward(grads);
            }
        }
    }
}
