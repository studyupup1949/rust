use crate::operation_record::OperationRecord;
use crate::variable::Variable;
use std::cell::RefCell;

#[derive(Debug, Default)]
pub struct Tape {
    pub(crate) operations: RefCell<Vec<OperationRecord>>,
}

impl Tape {
    #[inline]
    pub fn new() -> Self {
        Self {
            operations: RefCell::new(Vec::new()),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            operations: RefCell::new(Vec::with_capacity(capacity)),
        }
    }

    #[inline]
    pub fn create_variable(&self, value: f64) -> Variable {
        Variable {
            index: {
                let mut operations = self.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([(0, 0.0), (0, 0.0)]));
                count
            },
            tape: self,
            value,
        }
    }
}
