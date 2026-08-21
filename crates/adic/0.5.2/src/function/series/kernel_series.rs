use std::{
    ops::Mul,
    rc::Rc,
};

use itertools::Itertools;

use crate::{
    error::{AdicError, AdicResult},
    local_num::{LocalOne, LocalZero},
    mapping::{IndexedMapping, Mapping},
    sequence::Sequence,
};


#[derive(Clone)]
/// A series, adding together the terms of a [`Sequence`]
///
/// When evaluating, provide the kernel function: another sequence `kernel`.
/// Then the sum will be performed as
///
/// ```text
/// self.sequence_0 * kernel_0 + self.sequence_1 * kernel_1 + ...
/// ````
pub struct KernelSeries<'t, T>
where T: 't {
    sequence: Rc<dyn Sequence<Term=T> + 't>,
}

impl<'t, T> KernelSeries<'t, T> {

    /// Constructs a new `Series` with the given sequence
    pub fn new<S>(sequence: S) -> Self
    where S: Sequence<Term=T> + 't {
        Self {
            sequence: Rc::new(sequence),
        }
    }

    /// Term [`Sequence`] for this `KernelSeries`
    pub fn term_sequence(&self) -> Rc<dyn Sequence<Term=T> + 't> {
        self.sequence.clone()
    }

}


impl<T> std::fmt::Debug for KernelSeries<'_, T>
where T: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let term_str = match self.sequence.approx_num_terms() {
            Some(n) if n <= 5 => self.sequence.terms()
                .enumerate()
                .map(|(idx, t)| format!("{t:?} * k_{idx}"))
                .join(" + "),
            _ => self.sequence.terms()
                .enumerate()
                .take(5)
                .map(|(idx, t)| format!("{t:?} * k_{idx}"))
                .chain(std::iter::once("...".to_string()))
                .join(" + "),
        };
        f.debug_struct("Series")
            .field("sequence", &term_str)
            .finish()
    }
}

impl<T, S> Mapping<S> for KernelSeries<'_, T>
where S: Sequence<Term=T>, T: Clone + LocalZero + LocalOne + Mul<Output=T> {
    type Output = T;
    fn eval(&self, kernel: S) -> AdicResult<T> {

        // Evaluate s_0 k_0 + s_0 k_1 + ...

        if (!self.sequence.is_finite_sequence() && !kernel.is_finite_sequence()) {
            return Err(AdicError::InfiniteOperation);
        }

        let Some(zero) = kernel.term_local_zero() else {
            return Err(AdicError::Severe("`Series` kernel cannot be empty".to_string()));
        };

        Ok(self.sequence.clone().term_mul(kernel, false).terms().fold(zero, |acc, t| acc + t))

    }
}

impl<T, S> IndexedMapping<S> for KernelSeries<'_, T>
where S: Sequence<Term=T>, T: Clone + LocalZero + LocalOne + Mul<Output=T> {

    fn eval_finite(&self, kernel: S, num_terms: usize) -> AdicResult<T> {

        // Evaluate s_0 k_0 + s_0 k_1 + ... + s_n k_n

        let Some(zero) = kernel.term_local_zero() else {
            return Err(AdicError::Severe("`Series` kernel cannot be empty".to_string()));
        };

        Ok(self.sequence.clone().term_mul(kernel, false).terms().take(num_terms).fold(zero, |acc, t| acc + t))

    }
}



#[cfg(test)]
mod test {

    use crate::mapping::Mapping;
    use super::KernelSeries;

    #[test]
    fn kernel() {

        let series = KernelSeries::new(vec![1, 2, 3]);
        let kernel = vec![4, 5, 6, 7];
        assert_eq!(Ok(1 * 4 + 2 * 5 + 3 * 6), series.eval(kernel));
        let kernel = vec![4, 5];
        assert_eq!(Ok(1 * 4 + 2 * 5), series.eval(kernel));

    }

}
