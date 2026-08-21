use std::marker::PhantomData;

use crate::{
    error::AdicResult,
    mapping::{IndexedMapping, Mapping},
};


#[derive(Debug, Clone)]
/// Structure that represents the lazy composition of two functions
pub struct ComposedMapping<Input, F, G>
where F: Mapping<G::Output>, G: Mapping<Input> {
    outer_function: F,
    inner_function: G,
    _marker: PhantomData<Input>,
}

impl <Input, F, G> ComposedMapping<Input, F, G>
where F: Mapping<G::Output>, G: Mapping<Input> {

    /// Constructor for composed function
    pub fn new(outer: F, inner: G) -> Self {
        Self { outer_function: outer, inner_function: inner, _marker: PhantomData }
    }

    /// Split composed function back apart
    pub fn split(self) -> (F, G) {
        (self.outer_function, self.inner_function)
    }

}

impl <Input, F, G> Mapping<Input> for ComposedMapping<Input, F, G>
where F: Mapping<G::Output>, G: Mapping<Input> {
    type Output = F::Output;
    fn eval(&self, x: Input) -> AdicResult<Self::Output> {
        self.outer_function.eval(self.inner_function.eval(x)?)
    }
}

// eval_finite() may behave in unexpected ways. It will take the given number of terms for the outer function
// but will take all the terms for the inner function
impl <Input, F, G> IndexedMapping<Input> for ComposedMapping<Input, F, G>
where F: IndexedMapping<G::Output>, G: Mapping<Input> {
    /// Evaluates the inner function and uses it to evaluate a specified number of terms of the outer function
    fn eval_finite(&self, x: Input, num_terms: usize) -> AdicResult<F::Output> {
        self.outer_function.eval_finite(self.inner_function.eval(x)?, num_terms)
    }
}



#[cfg(test)]
mod tests {
    use assertables::assert_matches;
    use num::Rational32;

    use crate::{
        error::AdicError,
        mapping::{IndexedMapping, Mapping},
        Polynomial, PowerSeries,
    };

    #[test]
    fn eval() {
        // (x + 1) o (x + 2) => (x + 3)
        let f1 = PowerSeries::new(vec![1, 1]);
        let f2 = PowerSeries::new(vec![2, 1]);
        let composed = f1.compose(f2);
        let expected = PowerSeries::new(vec![3, 1]);

        assert_eq!(composed.eval(1), expected.eval(1));
        assert_eq!(composed.eval(10), expected.eval(10));

        // (x + 1/2) o (x + 1/4) => (x + 3/4)
        let f1 = PowerSeries::new(vec![Rational32::new(1, 2), 1.into()]);
        let f2 = PowerSeries::new(vec![Rational32::new(1, 4), 1.into()]);
        let composed = f1.compose(f2);
        let expected = PowerSeries::new(vec![Rational32::new(3, 4), 1.into()]);

        assert_eq!(composed.eval(Rational32::new(1, 2)), expected.eval(Rational32::new(1, 2)));

        // (x^2) o (x - 2) => (x^2 - 4x + 4)
        // (x - 2) o (x^2) => (x^2 - 2)
        let f1 = PowerSeries::new(vec![0, 0, 1]);
        let f2 = PowerSeries::new(vec![-2, 1]);
        let f1_f2 = f1.clone().compose(f2.clone());
        let f2_f1 = f2.compose(f1);
        assert_ne!(f1_f2.eval(2), f2_f1.eval(2));

        let f1_f2_expected = PowerSeries::new(vec![4, -4, 1]);
        assert_eq!(f1_f2.eval(2), f1_f2_expected.eval(2));

        let f2_f1_expected = PowerSeries::new(vec![-2, 0, 1]);
        assert_eq!(f2_f1.eval(2), f2_f1_expected.eval(2));
    }

    #[test]
    fn eval_error() {
        let f1 = PowerSeries::new(vec![0, 0, 1]);
        let f2 = PowerSeries::new(|i| i32::try_from(i).unwrap());
        let f1_f2 = f1.clone().compose(f2.clone());
        assert_matches!(f1_f2.eval(1), Err(AdicError::InfiniteOperation));

        let f2_f1 = f2.compose(f1);
        assert_matches!(f2_f1.eval(1), Err(AdicError::InfiniteOperation));
    }

    #[test]
    fn eval_finite() {
        // (|i| i) o (x + 2) [3]=> 0 + 1*(x + 2) + 2*(x + 2)^2 == 2x^2 + 9x + 10
        let f1 = PowerSeries::new(|i| i32::try_from(i).unwrap());
        let f2 = PowerSeries::new(vec![2, 1]);
        let composed = f1.compose(f2);
        let expected = PowerSeries::new(vec![10, 9, 2]);


        assert_eq!(composed.eval_finite(1, 3), expected.eval(1));
        assert_eq!(composed.eval_finite(10, 3), expected.eval(10));
    }

    #[test]
    fn eval_finite_outer_polynomial() {

        // x^2 + x + 1
        let f = Polynomial::new(vec![1, 1, 1]);
        // 2x^2 + 2x + 1
        let g = Polynomial::new(vec![1, 2, 2]);
        // (2x^2 + 2x + 1)^2 + (2x^2 + 2x + 1) + 1 ~=~ 4x^4 + 8x^3 + 10x^2 + 6x + 3
        let f_g = f.clone().compose(g.clone());
        let f_g_full_poly = Polynomial::new(vec![3, 6, 10, 8, 4]);
        // 2(x^2 + x + 1)^2 + 2(x^2 + x + 1) + 1 ~=~ 2x^4 + 4x^3 + 8x^2 + 6x + 5
        let g_f = g.clone().compose(f.clone());
        let g_f_full_poly = Polynomial::new(vec![5, 6, 8, 4, 2]);

        for input in (0..10) {
            // Full evaluation equal
            assert_eq!(f_g.eval(input), f_g_full_poly.eval(input));
            assert_eq!(g_f.eval(input), g_f_full_poly.eval(input));
            // Five terms equal
            assert_eq!(f_g.eval_finite(input, 5), f_g_full_poly.eval_finite(input, 5));
            assert_eq!(g_f.eval_finite(input, 5), g_f_full_poly.eval_finite(input, 5));
            // Four terms unequal
            if input != 0 {
                assert_ne!(f_g.eval_finite(input, 4), f_g_full_poly.eval_finite(input, 4));
                assert_ne!(g_f.eval_finite(input, 4), g_f_full_poly.eval_finite(input, 4));
            }
            // One term unequal
            assert_eq!(f_g.eval_finite(input, 1), Ok(1));
            assert_eq!(f_g_full_poly.eval_finite(input, 1), Ok(3));
            assert_eq!(g_f.eval_finite(input, 1), Ok(1));
            assert_eq!(g_f_full_poly.eval_finite(input, 1), Ok(5));
        }

    }

}
