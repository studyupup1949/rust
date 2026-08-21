use crate::operation_record::OperationRecord;
use crate::variable::Variable;
use std::cell::RefCell;

#[derive(Debug, Default)]
pub struct Tape {
    pub(crate) operations: RefCell<Vec<OperationRecord>>,
}

impl Tape {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            operations: RefCell::new(Vec::new()),
        }
    }

    #[inline]
    #[must_use]
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

    #[inline]
    pub fn create_variables_as_array<const N: usize>(&self, values: &[f64; N]) -> [Variable; N] {
        std::array::from_fn(|i| self.create_variable(values[i]))
    }

    #[inline]
    pub fn create_variables(&self, values: &[f64]) -> Vec<Variable> {
        values
            .iter()
            .map(|value| self.create_variable(*value))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::tape::Tape;

    #[test]
    fn test_create_variables_as_array() {
        let tape = Tape::new();
        const N: usize = 3;
        const VALUES: [f64; N] = [1.0, 2.0, 3.0];

        let variables = tape.create_variables_as_array(&VALUES);

        assert_eq!(variables.len(), N);

        for (i, variable) in variables.iter().enumerate() {
            assert_eq!(variable.value, VALUES[i]);

            assert!(std::ptr::eq(variable.tape, &tape));
        }

        let indices: Vec<_> = variables.iter().map(|var| var.index).collect();
        let unique_indices: std::collections::HashSet<_> = indices.iter().copied().collect();
        assert_eq!(indices.len(), unique_indices.len());
    }
}
