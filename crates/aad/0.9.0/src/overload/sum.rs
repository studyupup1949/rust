use crate::variable::Variable;
use num_traits::Zero;
use std::iter::Sum;
use std::ops::Add;

impl<'a, 'b, F> Sum<&'a Variable<'b, F>> for Variable<'b, F>
where
    for<'c> &'c Variable<'b, F>: Add<&'c Variable<'b, F>, Output = Variable<'b, F>>,
    Variable<'b, F>: Zero,
{
    #[inline]
    fn sum<I: Iterator<Item = &'a Variable<'b, F>>>(iter: I) -> Self {
        iter.fold(Variable::zero(), |acc, x| &acc + x)
    }
}

impl<'a, F> Sum<Variable<'a, F>> for Variable<'a, F>
where
    Variable<'a, F>: Zero,
{
    #[inline]
    fn sum<I: Iterator<Item = Variable<'a, F>>>(iter: I) -> Self {
        iter.fold(Variable::zero(), |acc, x| acc + x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tape::Tape;

    #[test]
    fn test_sum() {
        let tape = Tape::new();
        let values = [1.0, 2.0, 3.0];
        let variables = tape.create_variables(&values);

        let sum: Variable<f64> = variables.iter().sum();

        assert_eq!(sum.value, 6.0);
        assert!(std::ptr::eq(sum.index.unwrap().1, &tape));
    }

    #[test]
    fn test_sum_empty() {
        let tape = Tape::new();
        let values = [];
        let variables = tape.create_variables(&values);

        let sum: Variable<f64> = variables.iter().sum();

        assert_eq!(sum.value, 0.0);
        assert!(sum.index.is_none());
    }

    #[test]
    fn test_sum_empty2() {
        let tape = Tape::new();
        let values = [];
        let variables = tape.create_variables(&values);

        let sum: Variable<f64> = variables.iter().sum();

        let x = tape.create_variable(5.0);

        let y = sum + x;

        assert_eq!(y.value, 5.0);
        assert!(std::ptr::eq(y.index.unwrap().1, &tape));
    }
}
