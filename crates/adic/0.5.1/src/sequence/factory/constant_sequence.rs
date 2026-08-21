use std::iter::repeat;
use super::Sequence;


#[derive(Debug, Clone)]
/// Constant [`Sequence`]
pub struct ConstantSequence<T>
where T: Clone {
    value: T,
}

impl<T> ConstantSequence<T>
where T: Clone {
    /// Creates a new constant [`Sequence`](Sequence)
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> Sequence for ConstantSequence<T>
where T: Clone {
    type Term = T;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        Box::new(repeat(self.value.clone()))
    }
    fn approx_num_terms(&self) -> Option<usize> {
        None
    }
}
