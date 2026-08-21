use std::{
    ops::Mul,
    fmt::Display,
    iter::{once, repeat_n},
};
use itertools::Itertools;
use num::BigInt;
use crate::{
    error::AdicResult,
    local_num::{LocalInteger, LocalOne, LocalZero},
    mapping::{IndexedMapping, Mapping},
    normed::Normed,
};
use super::{euclid, factorization};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Polynomial
///
/// `a_0 + a_1 x + a_2 x^2 + ... + a_d x^d`
///
/// ```
/// # use adic::{mapping::Mapping, EAdic, Polynomial, QAdic, Variety, ZAdic};
/// let x_minus_1 = Polynomial::new(vec![-1, 1]);
/// assert_eq!("1x^1 + -1x^0", x_minus_1.to_string());
/// let squared = x_minus_1.clone() * x_minus_1;
/// assert_eq!("1x^2 + -2x^1 + 1x^0", squared.to_string());
/// assert_eq!(squared.eval(1), Ok(0));
/// assert_eq!(squared.eval(3), Ok(4));
///
/// let poly = Polynomial::<EAdic>::new_with_prime(5, vec![3, -4, 1]);
/// assert_eq!(Polynomial::new_with_prime(5, vec![-1, 1]) * Polynomial::new_with_prime(5, vec![-3, 1]), poly);
/// assert_eq!(
///     Ok(Variety::new(vec![
///         QAdic::from_integer(ZAdic::new_approx(5, 6, vec![1, 0, 0, 0, 0, 0])),
///         QAdic::from_integer(ZAdic::new_approx(5, 6, vec![3, 0, 0, 0, 0, 0])),
///     ])),
///     poly.variety(6)
/// );
/// ```
pub struct Polynomial <T>
where T: Clone + LocalZero {
    coefficients: Vec<T>,
}

impl<T> Polynomial<T>
where T: Clone + LocalZero {

    /// Create a polynomial with the given coefficients
    pub fn new(coefficients: Vec<T>) -> Self {

        let mut coefficients = coefficients;
        while coefficients.last().is_some_and(T::is_local_zero) {
            coefficients.pop();
        }

        Self {
            coefficients,
        }

    }

    /// Create a polynomial by converting coefficients to `BigInt`
    ///
    /// It is common to need arbitrary precision integers when working with `Polynomial`.
    /// E.g. when factoring or converting to monic form,
    ///  arbitrary precision may be needed even if the input and output are small numbers.
    pub fn new_big(coefficients: Vec<T>) -> Polynomial<BigInt>
    where T: Into<BigInt> {
        Polynomial::new(coefficients.into_iter().map(Into::into).collect())
    }

    /// Create a polynomial with a single term with degree `degree`
    pub fn monomial(degree: usize, coefficient: T) -> Self {
        if coefficient.is_local_zero() {
            Self::new(vec![])
        } else {
            let zero = coefficient.local_zero();
            let mut coefficients = vec![zero; degree];
            coefficients.push(coefficient);
            Self::new(coefficients)
        }
    }

    /// Returns the polynomial for finding the n-th root of `a`: `x^n - a`
    pub fn nth_root_polynomial(a: T, n: u32) -> Self
    where T: LocalOne + std::ops::Neg<Output=T> {

        let n = usize::try_from(n).expect("Cannot convert u32 to usize");
        let coefficients = if n.is_local_zero() {
            vec![]
        } else {
            let zero = a.local_zero();
            let one = a.local_one();
            once(-a)
                .chain(repeat_n(zero, n - 1))
                .chain(once(one))
                .collect::<Vec<_>>()
        };

        Self {
            coefficients,
        }

    }


    /// Degree of this polynomial
    pub fn degree(&self) -> Option<usize> {
        self.coefficients.iter().rposition(|a| !a.is_local_zero())
    }

    /// Lowest degree of this polynomial, e.g. for f = x^2 - x, `f.lowest_degree() == 1`
    pub fn lowest_degree(&self) -> Option<usize> {
        self.coefficients.iter().position(|a| !a.is_local_zero())
    }

    /// Is the polynomial a constant, including `f(x) = 0`
    pub fn is_constant(&self) -> bool {
        self.degree().is_none_or(|d| d == 0)
    }

    /// Iterator reference for the coefficients of this polynomial
    pub fn coefficients(&self) -> impl ExactSizeIterator<Item=&T> + DoubleEndedIterator {
        self.coefficients.iter()
    }

    /// Iterator for the coefficients of this polynomial
    pub fn into_coefficients(self) -> impl ExactSizeIterator<Item=T> + DoubleEndedIterator {
        self.coefficients.into_iter()
    }

    /// Return degree and leading coefficient simultaneously or None if zero
    pub fn degree_and_leading_coefficient(&self) -> Option<(usize, T)> {
        self.coefficients().enumerate().rfind(|c| !c.1.is_local_zero()).map(|(d, c)| (d, c.clone()))
    }

    /// Return the leading coefficient or None if zero
    pub fn leading_coefficient(&self) -> Option<T> {
        self.degree_and_leading_coefficient().map(|c| c.1)
    }


    #[must_use]
    /// Convert between `Polynomial`s of different types
    ///
    /// ```
    /// # use adic::Polynomial;
    /// let poly = Polynomial::new(vec![4, 6, 7]);
    /// assert_eq!(Polynomial::new(vec![16, 36, 49]), poly.map_coefficients(|c| c * c));
    /// ```
    pub fn map_coefficients<U, F>(self, f: F) -> Polynomial<U>
    where U: Clone + LocalZero, F: FnMut(T) -> U {
        Polynomial::<U>::new(self.into_coefficients().map(f).collect())
    }

    #[must_use]
    /// Multiply all coefficients by a value `m`
    ///
    /// ```
    /// # use adic::Polynomial;
    /// let poly = Polynomial::new(vec![4, 6, 7]);
    /// assert_eq!(Polynomial::new(vec![60, 90, 105]), poly.mul_coefficients(15));
    /// ```
    pub fn mul_coefficients(self, m: T) -> Self
    where T: Mul<Output=T> {
        self.map_coefficients(move |c| m.clone() * c)
    }

    #[must_use]
    /// Factor greatest common denominator from polynomial coefficients
    ///
    /// ```
    /// # use adic::Polynomial;
    /// let poly = Polynomial::new(vec![60, 90, 105]);
    /// assert_eq!(Polynomial::new(vec![4, 6, 7]), poly.factor_coefficients());
    /// ```
    pub fn factor_coefficients(self) -> Self
    where T: LocalInteger {
        if let Some(zero) = self.coefficients.first().map(LocalZero::local_zero) {
            let g = self.coefficients.iter().fold(zero, |acc, c| acc.gcd(c));
            self.map_coefficients(|c| c / g.clone())
        } else {
            self
        }
    }

    #[must_use]
    /// Factor common denominator and initial unit from polynomial coefficients
    ///
    /// This will transform it into a canonical form with (1) the coefficients' GCD
    ///  and (2) the integer unit of the initial coefficient factored out.
    /// I.e. it is as close to monic form as it can be without the coefficients becoming fractional.
    ///
    /// ```
    /// # use num::Rational32;
    /// # use adic::{traits::AdicPrimitive, EAdic, Polynomial};
    /// let poly = Polynomial::<EAdic>::new_with_prime(5, vec![-5, 110, 50]);
    /// assert_eq!(
    ///     Polynomial::new(vec![EAdic::new_repeating(5, vec![], vec![2]), EAdic::new(5, vec![1, 2]), EAdic::new(5, vec![0, 1])]),
    ///     poly.primitive_monic()
    /// );
    /// ```
    pub fn primitive_monic(self) -> Self
    where T: LocalInteger + Normed, T::Unit: LocalOne {
        let factored = self.factor_coefficients();
        if let Some(leading) = factored.leading_coefficient() {

            let leading_unit = leading.unit().expect("Polynomial should have nonzero leading unit");
            let unit_one = leading_unit.local_one();
            let lu = T::from_unit(leading_unit);
            let factored = factored.map_coefficients(|c| c / lu.clone());

            // Set the leading coefficient explicitly with norm and a unit of "one"
            let mut coeffs = factored.into_coefficients().collect::<Vec<_>>();
            if let Some(mc) = coeffs.last_mut() {
                *mc = T::from_norm_and_unit(leading.norm(), unit_one.local_one());
            }
            Polynomial::new(coeffs)

        } else {
            factored
        }
    }

    #[must_use]
    /// Greatest common divisor between polynomials
    ///
    /// Uses the polynomial extended GCD algorithm.
    /// We use pseudo-remainders to avoid unnecessary division.
    ///
    /// Note two things:
    /// 1. Our GCD is normalized with gcd-factored coefficients, but the sign/unit is not unique
    /// 2.  We define the GCD of a polynomial `f` with the zero polynomial as `f` itself; "zero" is divisible by all polynomials
    ///
    /// (1) can be addressed if the coefficient type implements [`Normed`].
    /// Simply call [`primitive_monic`](Self::primitive_monic) afterward.
    /// This will transform it into a canonical form with the *coefficient* GCD
    ///  and the (integer) unit of the initial coefficient factored out.
    /// I.e. it is as close to monic form as it can be without the coefficients becoming fractional.
    ///
    /// <div class="warning">
    ///
    /// Do not use this method for a `Polynomial::<ZAdic>`.
    /// That struct works differently and has dedicated `Polynomial` methods.
    /// Use `Polynomial::zadic_gcd` instead.
    ///
    /// </div>
    pub fn gcd(&self, other: &Self) -> Self
    where T: LocalInteger {
        // euclid::pseudo_rem_gcd(self.clone(), other.clone())
        // euclid::primitive_pseudo_rem_gcd(self.clone(), other.clone())
        euclid::factored_pseudo_rem_gcd(self.clone(), other.clone())
    }

    #[must_use]
    /// Greatest common divisor between polynomials
    ///
    /// Uses the polynomial extended GCD algorithm.
    /// We use pseudo-remainders to avoid unnecessary division.
    ///
    /// Note two things:
    /// 1. Our GCD is normalized to be in primitive monic form, with coefficients factored and then leading unit divided out
    /// 2.  We define the GCD of a polynomial `f` with the zero polynomial as `f` itself; "zero" is divisible by all polynomials
    ///
    /// <div class="warning">
    ///
    /// Do not use this method for a `Polynomial::<ZAdic>`.
    /// That struct works differently and has dedicated `Polynomial` methods.
    /// Use `Polynomial::zadic_gcd` instead.
    ///
    /// </div>
    pub fn primitive_gcd(&self, other: &Self) -> Self
    where T: LocalInteger + Normed, T::Unit: LocalOne {
        euclid::primitive_pseudo_rem_gcd(self.clone(), other.clone())
    }

    #[must_use]
    /// Split `Polynomial` into square-free polynomials to successive powers
    ///
    /// The polynomial coefficients may be large after calculation.
    /// If `T` impls [`Normed`], use [`primitive_monic`](Self::primitive_monic) afterward to reduce them,
    ///  or better, use [`primitive_square_free_decomposition`](Self::primitive_square_free_decomposition).
    ///
    /// `f(x) = a_1 a_2^2 a_3^3 ... a_d^d`
    /// `f.square_free_decomposition() ~> vec![a_1, a_2, a_3, ... a_d]` (up to a constant multiple)
    ///
    /// <div class="warning">
    ///
    /// Do not use this method for a `Polynomial::<ZAdic>`.
    /// That struct works differently and has dedicated `Polynomial` methods.
    /// Use `Polynomial::zadic_square_free_decomposition` instead.
    ///
    /// </div>
    pub fn raw_square_free_decomposition(&self) -> Vec<Self>
    where T: LocalInteger {
        factorization::raw_square_free_decomposition(self)
    }

    #[must_use]
    /// Split `Polynomial` into primitive-monic square-free polynomials to successive powers
    ///
    /// The polynomial coefficients are as close to monic as possible without becoming fractional
    ///
    /// `f(x) = a_1 a_2^2 a_3^3 ... a_d^d`
    /// `f.square_free_decomposition() ~> vec![a_1, a_2, a_3, ... a_d]` (up to a constant multiple)
    ///
    /// <div class="warning">
    ///
    /// Do not use this method for a `Polynomial::<ZAdic>`.
    /// That struct works differently and has dedicated `Polynomial` methods.
    /// Use `Polynomial::zadic_square_free_decomposition` instead.
    ///
    /// </div>
    pub fn primitive_square_free_decomposition(&self) -> Vec<Self>
    where T: LocalInteger + Normed, T::Unit: LocalOne {
        factorization::primitive_square_free_decomposition(self)
    }

}

impl <T, U> Mapping<U> for Polynomial<T>
where T: Clone + LocalZero, U: Clone + LocalZero + LocalOne + Mul<T, Output=U> {
    type Output = U;
    fn eval(&self, input: U) -> AdicResult<U> {
        let mut output = input.local_zero();
        let mut input_to_the_n = input.local_one();
        for coefficient in self.coefficients.clone() {
            output = output + input_to_the_n.clone() * coefficient;
            input_to_the_n = input_to_the_n * input.clone();
        }
        Ok(output)
    }
}

impl <T, U> IndexedMapping<U> for Polynomial<T>
where T: Clone + LocalZero, U: Clone + LocalZero + LocalOne + Mul<T, Output=U> {
    fn eval_finite(&self, input: U, num_terms: usize) -> AdicResult<U> {
        let mut output = input.local_zero();
        let mut input_to_the_n = input.local_one();
        for coefficient in self.coefficients.iter().take(num_terms).cloned() {
            output = output + input_to_the_n.clone() * coefficient;
            input_to_the_n = input_to_the_n * input.clone();
        }
        Ok(output)
    }
}

impl<T> Display for Polynomial<T>
where T: Clone + LocalZero + Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            self.coefficients
                .iter()
                .enumerate()
                .rev()
                .map(|(deg, coeff)| format!("{coeff}x^{deg}"))
                .join(" + ")
        )
    }
}
