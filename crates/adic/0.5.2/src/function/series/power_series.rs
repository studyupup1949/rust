use std::{
    ops::{Add, Mul, Neg, Sub},
    rc::Rc,
};

use num::Zero;

use crate::{
    error::AdicResult,
    local_num::{LocalOne, LocalZero},
    mapping::{Differentiable, IndexedMapping, Mapping},
    sequence::{factory as sequence_factory, Sequence},
    Polynomial,
};
use super::KernelSeries;


#[derive(Debug, Clone)]
/// A series with a power of x multiplied by a [`Sequence`] of coefficients
///
/// `a_0 * x^0 + a_1 * x^1 + a_2 * x^2 + ...`
///
/// ```
/// # use adic::{mapping::{IndexedMapping, Mapping}, traits::{AdicPrimitive, PrimedFrom}, PowerSeries, ZAdic};
/// let f = |_| 1.0;
/// let s = PowerSeries::new(f);
/// assert_eq!(s.eval_finite(0.5, 4), Ok(1.875));
///
/// let v = vec![1, -2, 1];
/// let s = PowerSeries::new(v);
/// assert_eq!(s.eval(3), Ok(1 - 2*3 + 3*3));
///
/// let f = |_| ZAdic::one(5);
/// let s = PowerSeries::new(f);
/// let evaluate_at = ZAdic::primed_from(5, 2);
/// let result = ZAdic::primed_from(5, 15);
/// assert_eq!(s.eval_finite(evaluate_at, 4), Ok(result));
/// ```
pub struct PowerSeries<'t, T>
where T: 't {
    kernel_series: KernelSeries<'t, T>,
}

impl<'t, T> PowerSeries<'t, T> {

    /// Constructs a new `PowerSeries` with the given coefficients
    pub fn new<S>(coefficients: S) -> Self
    where S: Sequence<Term=T> + 't {
        Self {
            kernel_series: KernelSeries::new(coefficients),
        }
    }

    /// Coefficient [`Sequence`] for this `PowerSeries`
    pub fn coefficients(&self) -> Rc<dyn Sequence<Term=T> + 't> {
        self.kernel_series.term_sequence()
    }

    /// Truncates the `PowerSeries` and returns a [`Polynomial`] with the given number of terms
    ///
    /// ```
    /// # use adic::{Polynomial, PowerSeries};
    /// let series = PowerSeries::new(|n| n * n + 3);
    /// let poly = Polynomial::new(vec![3, 4, 7, 12, 19]);
    /// assert_eq!(poly, series.truncate(5));
    pub fn truncate(&self, num_terms: usize) -> Polynomial<T>
    where T: Clone + LocalZero {
        Polynomial::new(self.coefficients().terms().take(num_terms).collect())
    }

}


impl<T> Mapping<T> for PowerSeries<'_, T>
where T: Clone + LocalZero + LocalOne + Mul<Output=T> {
    type Output = T;
    fn eval(&self, x: T) -> AdicResult<T> {
        let kernel = sequence_factory::power(x.local_one(), x, 1);
        self.kernel_series.eval(kernel)
    }
}

impl<T> IndexedMapping<T> for PowerSeries<'_, T>
where T: Clone + LocalZero + LocalOne + Mul<Output=T> {
    fn eval_finite(&self, x: T, num_terms: usize) -> AdicResult<T> {
        let kernel = sequence_factory::power(x.local_one(), x, 1);
        self.kernel_series.eval_finite(kernel, num_terms)
    }
}

impl<T> Add for PowerSeries<'_, T>
where T: Clone + Add<Output=T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        PowerSeries::new(self.coefficients().term_add(rhs.coefficients()))
    }
}

impl<T> Neg for PowerSeries<'_, T>
where T: Neg<Output=T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        PowerSeries::new(self.coefficients().term_map(T::neg))
    }
}

impl<T> Sub for PowerSeries<'_, T>
where T: Clone + Add<Output=T> + Neg<Output=T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        PowerSeries::new(self.coefficients().term_add((-rhs).coefficients()))
    }
}

impl<T> Mul for PowerSeries<'_, T>
where T: Clone + Add<Output=T> + Mul<Output=T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        PowerSeries::new(self.coefficients().foil_mul(rhs.coefficients()))
    }
}

impl<T> Differentiable for  PowerSeries<'_, T>
where T: Clone + LocalZero + LocalOne + Mul<Output=T> {
    type Output = Self;
    fn into_derivative(self) -> Self::Output {
        let first = self.coefficients().terms().next().clone();
        if let Some(first) = first {
            let one = first.local_one();
            let l = sequence_factory::linear(one);
            let new_coefficients = l.term_mul(self.coefficients(), false);
            PowerSeries::new(new_coefficients)
        } else {
            Self::zero()
        }
    }
}

impl<T> Zero for PowerSeries<'_, T>
where T: Clone + LocalZero {
    fn zero() -> Self {
        Self::new(vec![])
    }
    fn is_zero(&self) -> bool {
        // Note: this is not true in a general series, but here because of the nature of power series
        self.coefficients().is_finite_sequence() && self.coefficients().terms().all(|t| t.is_local_zero())
    }
}


#[cfg(test)]
mod tests{
    use assertables::assert_in_delta;

    use crate::{
        mapping::{IndexedMapping, Mapping},
        sequence::factory as sequence_factory,
        traits::{AdicPrimitive, PrimedFrom},
        Polynomial, PowerSeries, UAdic, ZAdic,
    };

    #[test]
    fn debug() {
        assert_eq!(
            format!("{:?}", PowerSeries::new(vec![1, 2, 3])),
            "PowerSeries { kernel_series: Series { sequence: \"1 * k_0 + 2 * k_1 + 3 * k_2\" } }"
        );
        assert_eq!(
            format!("{:?}", PowerSeries::new(vec![1, 2, 3, 4, 5, 6, 7])),
            "PowerSeries { kernel_series: Series { sequence: \"1 * k_0 + 2 * k_1 + 3 * k_2 + 4 * k_3 + 5 * k_4 + ...\" } }"
        );
    }

    // TODO: Reimplement after AdicPolynomial adjusted to use One + Zero
    #[test]
    fn truncate() {
        let v = vec![2, 0, 1];
        let series = PowerSeries::<UAdic>::new_with_prime(5, v.clone());
        assert_eq!(series.truncate(1), Polynomial::new_with_prime(5, vec![2]));
        assert_eq!(series.truncate(2), Polynomial::new_with_prime(5, vec![2]));
        assert_eq!(series.truncate(3), Polynomial::new_with_prime(5, v.clone()));
        assert_eq!(series.truncate(100), Polynomial::new_with_prime(5, v));

        let term = uadic!(5, [1]);
        let f = sequence_factory::constant(term.clone());
        let series = PowerSeries::new(f);
        assert_eq!(series.truncate(1), Polynomial::new(vec![term.clone()]));
        assert_eq!(series.truncate(2), Polynomial::new(vec![term.clone(); 2]));
        assert_eq!(series.truncate(10), Polynomial::new(vec![term.clone(); 10]));
    }

    #[test]
    fn geometric() {
        let f = |_| 1.0;
        let s = PowerSeries::new(f);
        assert_eq!(s.eval_finite(0.5, 4), Ok(1.875));

        let f = |_| 1;
        let s = PowerSeries::new(f);
        assert_eq!(s.eval_finite(2, 4), Ok(15));

        let f = |_| ZAdic::one(5);
        let s = PowerSeries::new(f);
        assert_eq!(
            s.eval_finite(ZAdic::primed_from(5, 2), 4),
            Ok(ZAdic::from(uadic!(5, [0, 3])))
        );
    }

    #[test]
    fn exp() {
        let factorial = vec![1.0, 1.0, 0.5, 0.167, 0.042, 0.008];
        let s = PowerSeries::new(factorial);
        assert_in_delta!(s.eval(1.0).unwrap(), 2.72, 0.01);
    }

    #[test]
    fn ln() {
        let f = |n| {
            if n == 0 { 0.0 }
            else if n % 2 == 0 {
                -1.0 / f64::from(u32::try_from(n).unwrap())
            } else {
                1.0 / f64::from(u32::try_from(n).unwrap())
            }
        };
        let s = PowerSeries::new(f);
        assert_in_delta!(s.eval_finite(0.5, 20).unwrap(), 0.405, 0.001);
    }

    #[test]
    fn add() {
        let f1 = PowerSeries::new(|_| 1);
        let f2 = PowerSeries::new(|_| 2);
        let v1 = PowerSeries::new(vec![1, 2, 4, 8]);
        let v2 = PowerSeries::new(vec![0, 2, 4]);

        let f1_f2 = f1.clone() + f2.clone();
        let f1_v1 = f1.clone() + v1.clone();
        let v1_v2 = v1.clone() + v2.clone();
        let v2_f2 = v2 + f2;

        assert_eq!(f1_f2.eval_finite(3, 4), Ok(120));
        assert_eq!(f1_v1.eval_finite(3, 4), Ok(299));
        assert_eq!(v1_v2.eval(3), Ok(301));
        assert_eq!(v2_f2.eval_finite(3, 4), Ok(122));

        // Power series plus zero evaluates to the same value
        let zero_f = PowerSeries::new(|_| 0);
        let zero_v = PowerSeries::new(vec![]);
        let f1_zero_f = f1.clone() + zero_f.clone();
        let f1_zero_v = f1.clone() + zero_v.clone();
        let v1_zero_f = v1.clone() + zero_f.clone();
        let v1_zero_v = v1.clone() + zero_v.clone();
        assert_eq!(f1_zero_f.eval_finite(3, 4), f1.eval_finite(3, 4));
        assert_eq!(f1_zero_v.eval_finite(3, 4), f1.eval_finite(3, 4));
        assert_eq!(v1_zero_f.eval_finite(3, 4), v1.eval_finite(3, 4));
        assert_eq!(v1_zero_v.eval(3), v1.eval(3));

        // Power series of adic numbers
        // (1 + x + x^2 + ...)
        let zf1 = PowerSeries::new(|_| ZAdic::one(5));
        // (1 - 2x + x^2)
        let zv1 = PowerSeries::new(vec![
            ZAdic::from(uadic!(5, [1])),
            ZAdic::from(eadic_neg!(5, [3])),
            ZAdic::from(uadic!(5, [1])),
        ]);

        let x = zadic_approx!(5, 3, [0, 1]);

        assert_eq!(zf1.eval_finite(x.clone(), 10), Ok(zadic_approx!(5, 3, [1, 1, 1])));
        assert_eq!(zv1.eval_finite(x.clone(), 10), Ok(zadic_approx!(5, 3, [1, 3])));

        let zf1_zv1 = zf1 + zv1;
        assert_eq!(zf1_zv1.eval_finite(x, 10), Ok(zadic_approx!(5, 3, [2, 4, 1])));
    }

    #[test]
    fn sub() {
        // 1 + x + x^2 + x^3
        let f1 = PowerSeries::new(|_| 1);
        // 2 + 2x + 2x^2 + 2x^3
        let f2 = PowerSeries::new(|_| 2);
        // 1 + 2x + 4x^2 + 8x^3
        let v1 = PowerSeries::new(vec![1, 2, 4, 8]);
        // 2x + 4x^2
        let v2 = PowerSeries::new(vec![0, 2, 4]);

        let f1_f2 = f1.clone() - f2.clone();
        let f1_v1 = f1.clone() - v1.clone();
        let v1_v2 = v1.clone() - v2.clone();
        let v2_f2 = v2 - f2;

        // (1 + x + x^2 + x^3) - (2 + 2x + 2x^2 + 2x^3)
        // -1 - x - x^2 - x^3
        assert_eq!(f1_f2.eval_finite(3, 4), Ok(-40));

        // (1 + x + x^2 + x^3) - (1 + 2x + 4x^2 + 8x^3)
        // -x - 3x^2 - 7x^3
        assert_eq!(f1_v1.eval_finite(3, 4), Ok(-219));

        // (1 + 2x + 4x^2 + 8x^3) - (2x + 4x^2)
        // 1 + 8x^3
        assert_eq!(v1_v2.eval(3), Ok(217));

        // (2x + 4x^2) - (2 + 2x + 2x^2 + 2x^3)
        // -2 + 2x^2 - 2x^3
        assert_eq!(v2_f2.eval_finite(3, 4), Ok(-38));

        // Power series minus zero evaluates to the same value
        let zero_f = PowerSeries::new(|_| 0);
        let zero_v = PowerSeries::new(vec![]);
        let f1_zero_f = f1.clone() - zero_f.clone();
        let f1_zero_v = f1.clone() - zero_v.clone();
        let v1_zero_f = v1.clone() - zero_f.clone();
        let v1_zero_v = v1.clone() - zero_v.clone();
        assert_eq!(f1_zero_f.eval_finite(3, 4), f1.eval_finite(3, 4));
        assert_eq!(f1_zero_v.eval_finite(3, 4), f1.eval_finite(3, 4));
        assert_eq!(v1_zero_f.eval_finite(3, 4), v1.eval_finite(3, 4));
        assert_eq!(v1_zero_v.eval(3), v1.eval(3));

        assert_eq!((f1.clone() - f1.clone()).eval_finite(3, 4), Ok(0));

        // Power series of adic numbers
        // (1 + x + x^2 + ...)
        let zf1 = PowerSeries::new(|_| ZAdic::one(5));
        // (1 - 2x + x^2)
        let zv1 = PowerSeries::new(vec![
            ZAdic::from(uadic!(5, [1])),
            ZAdic::from(eadic_neg!(5, [3])),
            ZAdic::from(uadic!(5, [1])),
        ]);

        let x = zadic_approx!(5, 3, [0, 1]);

        assert_eq!(zf1.eval_finite(x.clone(), 10), Ok(zadic_approx!(5, 3, [1, 1, 1])));
        assert_eq!(zv1.eval_finite(x.clone(), 10), Ok(zadic_approx!(5, 3, [1, 3])));

        let zf1_zv1 = zf1 - zv1;
        assert_eq!(zf1_zv1.eval_finite(x, 10), Ok(zadic_approx!(5, 3, [0, 3, 0])));
    }


    #[test]
    fn mul() {
        let f1 = PowerSeries::new(|_| 1);
        let f2 = PowerSeries::new(|_| 2);
        let v1 = PowerSeries::new(vec![1, 2, 4, 8]);
        let v2 = PowerSeries::new(vec![0, 2, 4]);

        let f1_f2 = f1.clone() * f2.clone();
        let f1_v1 = f1.clone() * v1.clone();
        let v1_v2 = v1.clone() * v2.clone();
        let v2_f2 = v2.clone() * f2.clone();

        // (1 + x + x^2 + x^3) * (2 + 2x + 2x^2 + 2x^3)
        // 2 + 4x + 6x^2 + 8x^3
        assert_eq!(f1_f2.eval_finite(3, 4), Ok(284));

        // (1 + x + x^2 + x^3) * (1 + 2x + 4x^2 + 8x^3)
        // (1 + 3x + 7x^2 + 15x^3)
        assert_eq!(f1_v1.eval_finite(3, 4), Ok(478));

        // (1 + 2x + 4x^2 + 8x^3) * (2x + 4x^2)
        // (2x + 8x^2 + 16x^3)
        assert_eq!(v1_v2.eval_finite(3, 4), Ok(510));

        assert_eq!(v1_v2.eval(3), Ok(10878));
        assert_eq!(v1.eval(3).unwrap() * v2.eval(3).unwrap(), v1_v2.eval(3).unwrap());

        // (2x + 4x^2) * (2 + 2x + 2x^2 + 2x^3)
        // (4x + 12x^2 + 12x^3)
        assert_eq!(v2_f2.eval_finite(3, 4), Ok(444));

        // Power series times zero evaluates to zero
        let zero_f = PowerSeries::new(|_| 0);
        let zero_v = PowerSeries::new(vec![]);
        let f1_zero_f = f1.clone() * zero_f.clone();
        let f1_zero_v = f1.clone() * zero_v.clone();
        let v1_zero_f = v1.clone() * zero_f.clone();
        let v1_zero_v = v1.clone() * zero_v.clone();
        assert_eq!(f1_zero_f.eval_finite(3, 4), Ok(0));
        assert_eq!(f1_zero_v.eval_finite(3, 4), Ok(0));
        assert_eq!(v1_zero_f.eval_finite(3, 4), Ok(0));
        assert_eq!(v1_zero_v.eval(3), Ok(0));

        // Power series times one evaluates to the same value
        let one_f = PowerSeries::new(|i| if i == 0 {1} else {0});
        let one_v = PowerSeries::new(vec![1]);
        let f1_one_f = f1.clone() * one_f.clone();
        let f1_one_v = f1.clone() * one_v.clone();
        let v1_one_f = v1.clone() * one_f.clone();
        let v1_one_v = v1.clone() * one_v.clone();
        assert_eq!(f1_one_f.eval_finite(3, 4), f1.eval_finite(3, 4));
        assert_eq!(f1_one_v.eval_finite(3, 4), f1.eval_finite(3, 4));
        assert_eq!(v1_one_f.eval_finite(3, 4), v1.eval_finite(3, 4));
        assert_eq!(v1_one_v.eval(3), v1.eval(3));

        // Power series of adic numbers
        // (1 + x + x^2 + ...)
        let zf1 = PowerSeries::new(|_| ZAdic::one(5));
        // (1 - 2x + x^2)
        let zv1 = PowerSeries::new(vec![
            ZAdic::from(uadic!(5, [1])),
            ZAdic::from(eadic_neg!(5, [3])),
            ZAdic::from(uadic!(5, [1])),
        ]);

        let x = zadic_approx!(5, 3, [0, 1]);

        assert_eq!(zf1.eval_finite(x.clone(), 10), Ok(zadic_approx!(5, 3, [1, 1, 1])));
        assert_eq!(zv1.eval_finite(x.clone(), 10), Ok(zadic_approx!(5, 3, [1, 3])));

        let zf1_zv1 = zf1 * zv1;
        assert_eq!(zf1_zv1.eval_finite(x, 10), Ok(zadic_approx!(5, 3, [1, 4, 4])));
    }
}
