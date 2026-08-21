use std::fmt::Display;
use std::iter::{once, repeat_n};
use itertools::Itertools;
use num::traits::Pow;
use crate::{
    adic_valid, polynomial_variety, variety_size,
    AdicNumber, AdicResult, IAdic, Prime, RAdic, SignedAdicNumber, UAdic, ZAdic, ZAdicVariety,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A polynomial with adic number coefficients
///  ([`iadic_poly`](crate::iadic_poly), [`zadic_poly`](crate::zadic_poly))
pub struct AdicPolynomial <T>
where T: AdicNumber {
    p: Prime,
    coefficients: Vec<T>,
}

impl<T> AdicPolynomial<T>
where T: AdicNumber {

    /// Prime for this adic polynomial
    pub fn p(&self) -> Prime {
        self.p
    }

    /// Degree of this polynomial
    pub fn degree(&self) -> Option<usize> {
        match self.coefficients.len() {
            0 => None,
            n => Some(n - 1)
        }
    }

    /// Lowest degree of this polynomial, e.g. for f = x^2 - x, `f.lowest_degree() == 1`
    pub fn lowest_degree(&self) -> Option<usize> {
        self.coefficients.iter().position(|a| !a.is_zero())
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
    pub fn new<P>(p: P, mut coefficients: Vec<T>) -> Self
    where P: Into<Prime> {

        let p = p.into();

        while coefficients.last().is_some_and(T::is_zero) {
            coefficients.pop();
        }

        coefficients.iter().for_each(|c| adic_valid::validate_mono_character(p, c.p()));

        Self {
            p,
            coefficients,
        }
    }

    /// Returns the polynomial for finding the n-th root of `a`: `x^n - a`
    pub fn nth_root_polynomial(a: T, n: u32) -> Self
    where T: SignedAdicNumber {

        let p = a.p();
        let n = usize::try_from(n).expect("Cannot convert u32 to usize");

        let coefficients = if n == 0 {
            vec![]
        } else {
            once(-a)
                .chain(repeat_n(T::zero(p), n - 1))
                .chain(once(T::one(p)))
                .collect::<Vec<_>>()
        };

        Self {
            p,
            coefficients,
        }

    }

    /// The derivative of this `AdicPolynomial`.
    /// See also: [`into_derivative`](Self::into_derivative)
    ///
    /// # Panics
    /// Panics if usize -> u32 conversion fails during power coefficient multiplication
    ///
    /// ```
    /// # use adic::{iadic_pos, AdicPolynomial};
    /// let poly = AdicPolynomial::new(5, vec![iadic_pos!(5, [3]), iadic_pos!(5, [2, 1]), iadic_pos!(5, [3])]);
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
                let deg_t = T::from_u32(self.p(), u32::try_from(deg).expect("derivative degree u32 conversion"));
                deg_t * coeff.clone()
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
    /// # use adic::{iadic_pos, AdicPolynomial};
    /// let poly = AdicPolynomial::new(5, vec![iadic_pos!(5, [3]), iadic_pos!(5, [2, 1]), iadic_pos!(5, [3])]);
    /// assert_eq!("3._5x^2 + 12._5x^1 + 3._5x^0", poly.to_string());
    /// let deriv = poly.into_derivative();
    /// assert_eq!("11._5x^1 + 12._5x^0", deriv.to_string());
    /// ```
    #[must_use]
    pub fn into_derivative(self) -> Self {
        let p = self.p();
        let new_coefficients = self.coefficients
            .into_iter()
            .enumerate()
            .skip(1)
            .map(|(deg, coeff)| {
                let deg_t = T::from_u32(p, u32::try_from(deg).expect("derivative degree u32 conversion"));
                deg_t * coeff
            })
            .collect();
        Self::new(self.p, new_coefficients)
    }

    /// Solve for the roots of `AdicPolynomial` and return the [`AdicVariety`](crate::AdicVariety)
    ///
    /// The algorithm works in two steps:
    /// - Find all solutions mod p^(2*v+1), where v is the valuation of f'(x) at each solution
    /// - Use the Hensel lift/Newton approximation to calculate the remaining digits
    ///
    /// Hensel's lemma basically says if the valuation of f(x) is more than twice that of f'(x),
    ///  then you are close to a simple root of the polynomial
    ///  and you can "lift" this approximate root of the polynomial to a full p-adic root.
    /// That is why there are two steps: first find an approximate root and then lift it to a true root.
    ///
    /// In the simplest case, you find solutions in F_p (solutions mod p) and "lift" those to F_p^2, F_p^3, etc.
    /// Currently, the first step is done semi-manually, trying each x in [0, 1, ... p-1] to see where it becomes zero.
    /// Then using those solutions, look for more solutions mod p^k.
    /// With the Newton-ish approximation of `f(y) = f(x) + f'(y-x) * (y-x)`, you plug in each new digit:
    /// - `f(r_k) = f(r_{k-1} + d * p^k) = f(r_{k-1}) + d * p^k * f'(r_{k-1}) mod p^{k+1}`
    /// - `f(r_{k-1}) = F p^k = 0 mod p^k`
    /// - `0 = f(r_k) = f(r) + d * p^k f'(r) = p^k (F + d * f'(r)) mod p^{k+1}`
    /// - `d = - F * (f'(r))^{-1} mod p`
    /// - `d = - (p^{-k} f(r_{k-1})) * (f'(r_{k-1}))^{-1} mod p`
    ///
    /// This gives the next digit d of the root from the last guess, `r_{k-1}`.
    /// If n has a factor of p, then the algorithm is more complicated because you have to take into account more digits.
    ///
    /// In the case that f(x) NEVER has valuation more than twice that of f'(x), Hensel's lemma fails.
    /// But in this case, the derivative also approaches zero, meaning this is actually a double root.
    /// We have plans to use this fact to also find degenerate roots, but currently the method will return an error in this case.
    ///
    /// 5-adic (x + 1)(x + 2) = 1 + 3 x + x^2
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use adic::{zadic_poly, zadic_variety};
    /// let p = 5;
    /// let f = zadic_poly!(p, [2, 3, 1]);
    /// let precision = 6;
    /// let expected = zadic_variety!(p, 6, [
    ///     [3, 4, 4, 4, 4, 4],
    ///     [4, 4, 4, 4, 4, 4],
    /// ]);
    /// let variety = f.variety(precision)?;
    /// assert_eq!(expected, variety);
    /// assert_eq!("variety(...444443._5, ...444444._5)", variety.to_string());
    /// # Ok(()) }
    /// ```
    ///
    /// 7-adic (x + 1)(x^2 - 2) = - 2 - 2 x + x^2 + x^3
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use adic::{zadic_poly, zadic_variety};
    /// let p = 7;
    /// let f = zadic_poly!(p, [-2, -2, 1, 1]);
    /// let precision = 6;
    /// let expected = zadic_variety!(p, 6, [
    ///     [3, 1, 2, 6, 1, 2],
    ///     [4, 5, 4, 0, 5, 4],
    ///     [6, 6, 6, 6, 6, 6],
    /// ]);
    /// let variety = f.variety(precision)?;
    /// assert_eq!(expected, variety);
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    /// 1. `AdicPolynomial`'s `certainty` is not high enough for desired `precision`
    /// 2. A degenerate root is suspected (multiplicity not yet supported but on its way)
    ///
    /// # Panics
    /// Panics if certainty does not behave as expected
    ///
    /// <div class="warning">
    ///
    /// We currently cannot handle degenerate roots, e.g. `f(x) = (x+2)^2`.
    /// The method will return an error in this case.
    /// We are working to rectify this.
    ///
    /// </div>
    ///
    /// <div class="warning">
    ///
    /// This currently returns varieties of adic INTEGERS.
    /// If the root of the polynomial is fractional, it will TRUNCATE that fraction.
    /// E.g. The 5-adic root of (5 x - 1) will be `...000._5` instead of `...000.1_5`.
    /// This method will default to fractional output in the near future.
    ///
    /// </div>
    pub fn variety(&self, precision: usize) -> AdicResult<ZAdicVariety>
    where T: Into<ZAdic>, Self: Into<AdicPolynomial<ZAdic>> {
        polynomial_variety(self.clone(), precision)
    }

    /// Return the size of the variety for this `AdicPolynomial`: the number of simple roots
    ///
    /// 7-adic (x + 1)(x^2 - 2)(x^2 - 3) = 6 + 6 x - 5 x^2 - 5 x^3 + x^4 + x^5
    ///
    /// Expect size 3 because -1 and +/- sqrt(2) exist in 7-adics but not +/- sqrt(3).
    /// ```
    /// use adic::{zadic_poly, zadic_variety};
    /// let p = 7;
    /// let f = zadic_poly!(p, [6, 6, -5, -5, 1, 1]);
    /// assert_eq!(Ok(3), f.variety_size());
    /// ```
    ///
    /// # Errors
    /// Errors if rootfinding encounters problems, e.g. heavily degenerate roots
    pub fn variety_size(&self) -> AdicResult<usize>
    where Self: Into<AdicPolynomial<ZAdic>> {
        variety_size(self)
    }


    /// Evaluates the `AdicPolynomial` at the given input value
    ///
    /// # Panics
    /// Panics if usize to u32 conversion fails
    pub fn evaluate(&self, input: &T) -> T
    where T: Pow<u32, Output = T> {
        self.coefficients
        .iter()
        .enumerate()
        .map(|(degree, a)| a.clone() * input.clone().pow(u32::try_from(degree).expect("usize to u32 conversion")))
        .fold(T::zero(self.p), |acc, x| acc + x )
    }

}


impl From<AdicPolynomial<UAdic>> for AdicPolynomial<ZAdic>  {
    fn from(f: AdicPolynomial<UAdic>) -> Self {
        let p = f.p();
        let coefficients = f.into_coefficients().map(ZAdic::from).collect();
        Self::new(p, coefficients)
    }
}

impl From<AdicPolynomial<IAdic>> for AdicPolynomial<ZAdic>  {
    fn from(f: AdicPolynomial<IAdic>) -> Self {
        let p = f.p();
        let coefficients = f.into_coefficients().map(ZAdic::from).collect();
        Self::new(p, coefficients)
    }
}

impl From<AdicPolynomial<RAdic>> for AdicPolynomial<ZAdic>  {
    fn from(f: AdicPolynomial<RAdic>) -> Self {
        let p = f.p();
        let coefficients = f.into_coefficients().map(ZAdic::from).collect();
        Self::new(p, coefficients)
    }
}


impl<T> Display for AdicPolynomial<T>
where T: Display + AdicNumber {
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
    use crate::{iadic_poly, iadic_neg, iadic_pos, radic, zadic_exact, zadic_variety, AdicNumber, AdicPolynomial, IAdic};

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
    fn degree() {
        assert_eq!(iadic_poly!(5, [-2, 0, 1]).degree(), Some(2));
        assert_eq!(iadic_poly!(5, [-1, 1]).degree(), Some(1));
        assert_eq!(iadic_poly!(5, [1]).degree(), Some(0));
        assert_eq!(AdicPolynomial::<IAdic>::new(5, vec![]).degree(), None);
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
            zadic_exact!(iadic_pos!(5, [1])),
            zadic_exact!(iadic_pos!(5, [])),
            zadic_exact!(iadic_pos!(5, [1])),
        ]).derivative();
        let expected = AdicPolynomial::new(5, vec![
            zadic_exact!(iadic_pos!(5, [])),
            zadic_exact!(iadic_pos!(5, [2])),
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