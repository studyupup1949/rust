use crate::operation_record::OperationRecord;
use crate::variable::Variable;
use num_traits::Zero;
use std::cell::RefCell;
use std::fmt::Debug;

#[derive(Debug, Default)]
pub struct Tape<F: Sized> {
    pub(crate) operations: RefCell<Vec<OperationRecord<F>>>,
}

impl<F> Tape<F> {
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
}

impl<F: Copy + Zero> Tape<F> {
    #[inline]
    pub fn create_variable(&self, value: F) -> Variable<F> {
        Variable {
            index: {
                let mut operations = self.operations.borrow_mut();
                let count = (*operations).len();
                (*operations).push(OperationRecord([
                    (usize::MAX, F::zero()),
                    (usize::MAX, F::zero()),
                ]));
                Some((count, self))
            },
            value,
        }
    }

    #[inline]
    pub fn create_variables<const N: usize>(&self, values: &[F; N]) -> [Variable<F>; N] {
        std::array::from_fn(|i| self.create_variable(values[i]))
    }

    #[inline]
    pub fn create_variables_iter(&self, values: &[F]) -> impl Iterator<Item = Variable<F>> {
        values.iter().map(|value| self.create_variable(*value))
    }
}

#[cfg(test)]
mod tests {
    use crate::tape::Tape;

    #[test]
    fn test_create_variables() {
        let tape = Tape::new();
        const N: usize = 3;
        const VALUES: [f64; N] = [1.0, 2.0, 3.0];

        let variables = tape.create_variables(&VALUES);

        assert_eq!(variables.len(), N);

        for (i, variable) in variables.iter().enumerate() {
            assert_eq!(variable.value, VALUES[i]);

            assert!(std::ptr::eq(variable.index.unwrap().1, &tape));
        }
    }
}
