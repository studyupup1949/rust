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
                (*operations).push(OperationRecord([(0, F::zero()), (0, F::zero())]));
                count
            },
            tape: Some(self),
            value,
        }
    }

    #[inline]
    pub fn create_variables<const N: usize>(&self, values: &[F; N]) -> [Variable<F>; N] {
        std::array::from_fn(|i| self.create_variable(values[i]))
    }

    #[inline]
    pub fn create_variables_vec(&self, values: &[F]) -> Vec<Variable<F>> {
        values
            .iter()
            .map(|value| self.create_variable(*value))
            .collect()
    }
}

impl<F: Debug + Zero + PartialEq> Tape<F> {
    /// Converts the computation graph to DOT format for visualization.
    ///
    /// This method generates a DOT language representation of the computation graph,
    /// which can be used with tools like Graphviz to create visual diagrams.
    ///
    /// # Returns
    ///
    /// A string containing the DOT representation of the graph where:
    /// - Variables (leaf nodes) are shown as green boxes
    /// - Operations (non-leaf nodes) are shown as circles
    /// - Edges are labeled with their gradient values
    ///
    /// # Example
    ///
    /// ```
    /// use aad::tape::Tape;
    /// let tape = Tape::<f64>::new();
    /// let x = tape.create_variable(2.0);
    /// let y = x * x;
    /// let dot = tape.to_dot();
    /// // The resulting DOT string can be used with Graphviz
    /// ```
    pub fn to_dot(&self) -> String {
        let operations = self.operations.borrow();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for (i, op) in operations.iter().enumerate() {
            let is_leaf = op.0.iter().all(|(_, grad)| grad.is_zero());

            if is_leaf {
                nodes.push(format!(
                    "var{i} [label=\"Variable {i}\\n(index: {i})\", color=green, shape=box];"
                ));
            } else {
                nodes.push(format!("var{i} [label=\"Operation {i}\\n(index: {i})\"];"));
            }

            if !is_leaf {
                for (input_idx, grad) in &op.0 {
                    if !grad.is_zero() {
                        edges.push(format!("var{input_idx} -> var{i} [label=\"{grad:?}\"];"));
                    }
                }
            }
        }

        format!(
            "digraph ComputationGraph {{\n\t{}\n\t{}\n}}",
            nodes.join("\n\t"),
            edges.join("\n\t")
        )
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

            assert!(std::ptr::eq(variable.tape.unwrap(), &tape));
        }

        let indices: Vec<_> = variables.iter().map(|var| var.index).collect();
        let unique_indices: std::collections::HashSet<_> = indices.iter().copied().collect();
        assert_eq!(indices.len(), unique_indices.len());
    }
}
