use std::fmt::Display;
use itertools::Itertools;
use crate::{adic_valid, polynomial_variety, AdicError, AdicInteger, ZAdicVariety};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A polynomial with adic integer coefficients
pub struct AdicPolynomial <T: AdicInteger> {
    p: u32,
    coefficients: Vec<T>,
}

impl<T: AdicInteger> AdicPolynomial<T> {

    /// Prime for this adic polynomial
    pub fn p(&self) -> u32 {
        self.p
    }

    /// Order of this polynomial
    pub fn order(&self) -> usize {
        self.coefficients.len()
    }

    /// Iterator reference for the coefficients of this polynomial
    pub fn coefficients(&self) -> impl Iterator<Item=&T> {
        self.coefficients.iter()
    }

    /// Iterator for the coefficients of this polynomial
    pub fn into_coefficients(self) -> impl Iterator<Item=T> {
        self.coefficients.into_iter()
    }

    /// Create an adic polynomial with the given coefficients
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new(p: u32, mut coefficients: Vec<T>) -> Self {

        adic_valid::validate_p(p);

        while coefficients.last().is_some_and(AdicInteger::is_zero) {
            coefficients.pop();
        }

        coefficients.iter().for_each(|c| adic_valid::validate_mono_character(p, c.p()));

        Self {
            p,
            coefficients,
        }
    }
}

impl<T> AdicPolynomial<T>
where T: AdicInteger, u32: std::ops::Mul<T, Output=T> {

    /// The derivative of this `AdicPolynomial`.
    /// See also: [`into_derivative`](Self::into_derivative)
    ///
    /// # Panics
    /// Panics if usize -> u32 conversion fails during power coefficient multiplication
    ///
    /// ```
    /// # use adic::{uadic, AdicPolynomial};
    /// let poly = AdicPolynomial::new(5, vec![uadic!(5, [3]), uadic!(5, [2, 1]), uadic!(5, [3])]);
    /// assert_eq!("3._5x^2 + 12._5x^1 + 3._5x^0", poly.to_string());
    /// let deriv = poly.derivative();
    /// assert_eq!("11._5x^1 + 12._5x^0", deriv.to_string());
    /// ```
    #[must_use]
    pub fn derivative(&self) -> Self {
        let new_coefficients = self.coefficients
            .iter()
            .enumerate()
            .skip(1)
            .map(|(deg, coeff)| {
                let deg_u32 = u32::try_from(deg).expect("derivative degree u32 conversion");
                deg_u32 * coeff.clone()
            })
            .collect();
        Self::new(self.p, new_coefficients)
    }

    /// Consume `AdicPolynomial` and get the derivative
    ///
    /// # Panics
    /// Panics if usize -> u32 conversion fails during power coefficient multiplication
    ///
    /// ```
    /// # use adic::{uadic, AdicPolynomial};
    /// let poly = AdicPolynomial::new(5, vec![uadic!(5, [3]), uadic!(5, [2, 1]), uadic!(5, [3])]);
    /// assert_eq!("3._5x^2 + 12._5x^1 + 3._5x^0", poly.to_string());
    /// let deriv = poly.into_derivative();
    /// assert_eq!("11._5x^1 + 12._5x^0", deriv.to_string());
    /// ```
    #[must_use]
    pub fn into_derivative(self) -> Self {
        let new_coefficients = self.coefficients
            .into_iter()
            .enumerate()
            .skip(1)
            .map(|(deg, coeff)| {
                let deg_u32 = u32::try_from(deg).expect("derivative degree u32 conversion");
                deg_u32 * coeff
            })
            .collect();
        Self::new(self.p, new_coefficients)
    }

}

impl<T> AdicPolynomial<T>
where T: AdicInteger + std::ops::Neg<Output=T>,
    u32: std::ops::Mul<T, Output=T> {

    /// Solve for the roots of `AdicPolynomial` and return the `ZAdicVariety`
    ///
    /// # Errors
    /// Errors if:
    /// 1. Polynomial is not of the form `x^n - a`
    /// 2. n == 0
    /// 3. `precision` is not high enough (roughly, `self.certainty() >= n * precision`)
    ///
    /// # Panics
    /// Panics if polynomial is exactly zero
    ///
    /// <div class="warning">
    ///
    /// Currently, this returns an error if self is not in the form [a, 0, 0, ..., 1], i.e. an n-th root.
    /// We do not handle general polynomials at this point.
    /// This just serves as a passthrough to an adic integer's [`nth_root`](AdicInteger::nth_root).
    ///
    /// </div>
    pub fn variety(&self, precision: usize) -> Result<ZAdicVariety, AdicError> {
        polynomial_variety(self.p, self.coefficients.as_slice(), precision)
    }

}

impl<T: AdicInteger> Display for AdicPolynomial<T> {
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



#[cfg(test)]
mod tests {
    use crate::{iadic_neg, iadic_pos, radic, zadic_exact_pos, zadic_variety, AdicPolynomial, AdicInteger, IAdic};

    #[test]
    fn new() {
        AdicPolynomial::new(5, vec![IAdic::one(5)]);
    }

    #[test]
    #[should_panic(expected="MixedCharacteristic")]
    fn mismatched_p() {
        AdicPolynomial::new(3, vec![IAdic::one(5)]);
    }

    #[test]
    #[should_panic(expected="4 is not prime")]
    fn composite_p() {
        AdicPolynomial::<IAdic>::new(4, vec![]);
    }

    #[test]
    fn derivative() {
        let actual = AdicPolynomial::new(5, vec![
            iadic_pos!(5, [1]),
            iadic_neg!(5, [2]),
            iadic_pos!(5, [1]),
        ]).derivative();
        let expected = AdicPolynomial::new(5, vec![
            iadic_neg!(5, [2]),
            iadic_pos!(5, [2]),
        ]);
        assert_eq!(actual, expected);

        let actual = AdicPolynomial::new(5, vec![
            radic!(5, [1], []),
            radic!(5, [], []),
            radic!(5, [4], [3]),
            radic!(5, [1], []),
        ]).derivative();
        let expected = AdicPolynomial::new(5, vec![
            radic!(5, [], []),
            radic!(5, [3], [2]),
            radic!(5, [3], []),
        ]);
        assert_eq!(actual, expected);

        let actual = AdicPolynomial::new(5, vec![
            zadic_exact_pos!(5, [1]),
            zadic_exact_pos!(5, []),
            zadic_exact_pos!(5, [1]),
        ]).derivative();
        let expected = AdicPolynomial::new(5, vec![
            zadic_exact_pos!(5, []),
            zadic_exact_pos!(5, [2]),
        ]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn variety() {
        let expected = zadic_variety!(7, 3, [
            [3, 1, 2],
            [4, 5, 4],
        ]);
        let actual = AdicPolynomial::new(7, vec![
            -iadic_pos!(7, [2]),
            IAdic::zero(7),
            IAdic::one(7),
        ]).variety(3).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn display() {
        let expected = "1._5x^2 + 0._5x^1 + (4)._5x^0".to_string();
        let actual = AdicPolynomial::new(5, vec![radic!(5, [], [4]), radic!(5, [], []), radic!(5, [1], [])]).to_string();

        assert_eq!(expected, actual);
    }
}