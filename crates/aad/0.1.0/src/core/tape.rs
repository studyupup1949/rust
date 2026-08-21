use crate::core::operations::Operation;
use std::cell::UnsafeCell;
use crate::core::var::Var;

#[derive(Default)]
pub struct Tape {
    operations: UnsafeCell<Vec<Operation>>,
    pub(crate) values: UnsafeCell<Vec<f64>>,
    pub(crate) grads: UnsafeCell<Vec<f64>>,
}

impl Tape {
    pub fn var(&self, value: f64) -> Var {
        unsafe {
            let idx = (*self.values.get()).len();
            (*self.values.get()).push(value);
            Var { idx, tape: self }
        }
    }

    pub(crate) fn record(&self, operation: Operation) {
        unsafe {
            (*self.operations.get()).push(operation);
        }
    }

    pub(crate) fn replay(&self) {
        unsafe {
            let ops = &(*self.operations.get());
            for operation in ops.iter().rev() {
                operation.backward(self);
            }
        }
    }
}
