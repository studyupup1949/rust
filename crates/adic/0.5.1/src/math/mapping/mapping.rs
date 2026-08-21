use crate::error::AdicResult;
use super::ComposedMapping;


/// Trait for functions
pub trait Mapping<Input> {

    /// Output for this `Mapping`
    type Output;

    /// Evaluates the `Mapping` at the given value `x`
    ///
    /// ```
    /// # use adic::{mapping::Mapping, PowerSeries};
    /// let series = PowerSeries::new(vec![1, 2, 3, 4, 5]);
    /// assert_eq!(series.eval(5), Ok(1 + 2 * 5 + 3 * 25 + 4 * 125 + 5 * 625));
    /// ```
    fn eval(&self, x: Input) -> AdicResult<Self::Output>;

    /// Creates a composed function `self o inner`
    ///
    /// ```
    /// # use adic::{mapping::Mapping, PowerSeries};
    /// // (5x + 1) o (x + 2) => (5x + 11)
    /// let f1 = PowerSeries::new(vec![1, 5]);
    /// let f2 = PowerSeries::new(vec![2, 1]);
    /// let composed = f1.compose(f2);
    /// let expected = PowerSeries::new(vec![11, 5]);
    /// assert_eq!(composed.eval(1), expected.eval(1));
    /// assert_eq!(composed.eval(10), expected.eval(10));
    /// ```
    fn compose<F, CInput>(self, inner: F) -> ComposedMapping<CInput, Self, F>
    where F: Mapping<CInput, Output=Input>, Self: Sized {
        ComposedMapping::new(self, inner)
    }

}


/// Trait for evaluating finite, term-based functions
pub trait IndexedMapping<Input>: Mapping<Input>{

    /// Evaluates the `Mapping` at the given value `x`, to a given number of terms
    /// ```
    /// # use adic::{mapping::IndexedMapping, PowerSeries};
    /// let series = PowerSeries::new(vec![1, 2, 3, 4, 5]);
    /// assert_eq!(series.eval_finite(5, 3), Ok(1 + 2 * 5 + 3 * 25));
    /// ```
    fn eval_finite(&self, x: Input, num_terms: usize) -> AdicResult<Self::Output>;

}



// Implementations

impl<In, Out, F> Mapping<In> for F
where F: Fn(In) -> Out {
    type Output = Out;
    fn eval(&self, x: In) -> AdicResult<Out> {
        Ok(self(x))
    }
}

#[cfg(test)]
mod tests {
    use super::Mapping;
    #[test]
    fn eval(){
        let f =|x| x * x;
        assert_eq!(f.eval(2), Ok(4));
        let g = f.compose(f);
        assert_eq!(g.eval(2), Ok(16));
    }

}
