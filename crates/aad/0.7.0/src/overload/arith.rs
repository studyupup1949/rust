use crate::operation_record::OperationRecord;
use crate::variable::Variable;
use num_traits::{Float, Inv, One, Zero};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl<F: Neg<Output = F> + One + Zero> Neg for Variable<'_, F> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        match self.tape {
            Some(tape) => Variable {
                index: {
                    let operations = &mut tape.operations.borrow_mut();
                    let count = (*operations).len();
                    (*operations).push(OperationRecord([
                        (self.index, F::one().neg()),
                        (0, F::zero()),
                    ]));
                    count
                },
                tape: self.tape,
                value: self.value.neg(),
            },
            None => Variable {
                index: 0,
                tape: None,
                value: self.value.neg(),
            },
        }
    }
}

impl<F: Add<F, Output = F> + One> Add<Self> for Variable<'_, F> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let tape = self.tape.or(rhs.tape);
        match tape {
            Some(tape) => Variable {
                index: {
                    let operations = &mut tape.operations.borrow_mut();
                    let count = (*operations).len();
                    (*operations).push(OperationRecord([
                        (self.index, F::one()),
                        (rhs.index, F::one()),
                    ]));
                    count
                },
                tape: Some(tape),
                value: self.value + rhs.value,
            },
            None => Variable {
                index: 0,
                tape: None,
                value: self.value + rhs.value,
            },
        }
    }
}

impl<F: Sub<F, Output = F> + One + Neg<Output = F>> Sub<Self> for Variable<'_, F> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let tape = self.tape.or(rhs.tape);
        match tape {
            Some(tape) => Variable {
                index: {
                    let operations = &mut tape.operations.borrow_mut();
                    let count = (*operations).len();
                    (*operations).push(OperationRecord([
                        (self.index, F::one()),
                        (rhs.index, -F::one()),
                    ]));
                    count
                },
                tape: Some(tape),
                value: self.value - rhs.value,
            },
            None => Variable {
                index: 0,
                tape: None,
                value: self.value - rhs.value,
            },
        }
    }
}

impl<F: Mul<F, Output = F> + Copy> Mul<Self> for Variable<'_, F> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let tape = self.tape.or(rhs.tape);
        match tape {
            Some(tape) => Variable {
                index: {
                    let operations = &mut tape.operations.borrow_mut();
                    let count = (*operations).len();
                    (*operations).push(OperationRecord([
                        (self.index, rhs.value),
                        (rhs.index, self.value),
                    ]));
                    count
                },
                tape: Some(tape),
                value: self.value * rhs.value,
            },
            None => Variable {
                index: 0,
                tape: None,
                value: self.value * rhs.value,
            },
        }
    }
}

impl<F: Copy + Div<F, Output = F> + Inv<Output = F> + Neg<Output = F> + Mul<Output = F>> Div<Self>
    for Variable<'_, F>
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.apply_binary_function(rhs, |x, y| x / y, |x, y| (y.inv(), -x / (y * y)))
    }
}

impl<F: Copy + One + Zero> AddAssign<Self> for Variable<'_, F> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<F: Copy + One + Neg<Output = F> + Sub<F, Output = F>> SubAssign<Self> for Variable<'_, F> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<F: Copy + Mul<F, Output = F>> MulAssign<Self> for Variable<'_, F> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<F: Copy + Div<Self, Output = Self> + Float + Inv<Output = F>> DivAssign<Self>
    for Variable<'_, F>
{
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<F: Copy + Zero + Add<F, Output = F> + One> Sum for Variable<'_, F> {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut iter = iter;
        let init_var = match iter.next() {
            None => {
                return Variable {
                    index: 0,
                    tape: None,
                    value: F::zero(),
                }
            }
            Some(x) => x,
        };

        iter.fold(init_var, |acc, x| acc + x)
    }
}

impl<'a, 'b, F: Copy + Zero + Add<F, Output = F> + One> Sum<&'b Variable<'a, F>>
    for Variable<'a, F>
{
    #[inline]
    fn sum<I: Iterator<Item = &'b Variable<'a, F>>>(iter: I) -> Self {
        let mut iter = iter;
        let init_var = match iter.next() {
            None => {
                return Variable {
                    index: 0,
                    tape: None,
                    value: F::zero(),
                }
            }
            Some(x) => x,
        };

        iter.fold(*init_var, |acc, x| acc + *x)
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

        let sum: Variable<f64> = variables.into_iter().sum();

        assert_eq!(sum.value, 6.0);
        assert!(std::ptr::eq(sum.tape.unwrap(), &tape));
    }

    #[test]
    fn test_sum_empty() {
        let tape = Tape::new();
        let values = [];
        let variables = tape.create_variables(&values);

        let sum: Variable<f64> = variables.into_iter().sum();

        assert_eq!(sum.value, 0.0);
        assert!(sum.tape.is_none());
    }

    #[test]
    fn test_sum_empty2() {
        let tape = Tape::new();
        let values = [];
        let variables = tape.create_variables(&values);

        let sum: Variable<f64> = variables.into_iter().sum();

        let x = tape.create_variable(5.0);

        let y = sum + x;

        assert_eq!(y.value, 5.0);
        assert!(std::ptr::eq(y.tape.unwrap(), &tape));
    }
}
