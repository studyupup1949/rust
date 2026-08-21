use crate::operations::Operation;
use crate::var::Var;
use std::cell::UnsafeCell;

#[derive(Debug, Default)]
pub struct Tape {
    pub(crate) operations: UnsafeCell<Vec<Operation>>,
}

impl Tape {
    #[inline]
    pub fn var(&self, value: f64) -> Var {
        unsafe {
            Var {
                idx: {
                    let operations = self.operations.get();
                    let count = (*operations).len();
                    (*operations).push(Operation([(0, 0.0), (0, 0.0)]));
                    count
                },
                tape: self,
                value,
            }
        }
    }
}
