use crate::{
    algorithm::{qadic_polynomial_variety, qadic_polynomial_variety_size},
    divisible::Prime,
    error::{AdicError, AdicResult},
    local_num::{LocalOne, LocalZero},
    traits::{AdicInteger, AdicPrimitive, PrimedFrom, PrimedInto},
    QAdic, Variety, ZAdic,
};
use super::{euclid, factorization, zadic_wrapper, Polynomial};


impl<A> Polynomial<A>
where A: AdicPrimitive {

    /// Create an adic polynomial by priming the given unprimed `coefficients` with prime `p`
    pub fn new_with_prime<P, N>(p: P, coefficients: Vec<N>) -> Self
    where P: Into<Prime>, N: PrimedInto<A> {
        let p = p.into();
        let primed_coefficients = coefficients.into_iter().map(|n| n.primed_into(p)).collect();
        Self::new(primed_coefficients)
    }

    /// Prime for this adic polynomial
    pub fn p(&self) -> AdicResult<Prime> {
        self.coefficients().next().map(A::p).ok_or(AdicError::NoPrimeSet)
    }

}

impl<A> Polynomial<A>
where A: AdicInteger, QAdic<A>: std::ops::Div<Output=QAdic<A>> {

    // Note: We should be able to do something similar for approximate adics, but for now only exact

    /// Monic polynomial
    ///
    /// The leading term will have coefficient `1` except for the zero polynomial.
    ///
    /// E.g. 5-adic `-50x^2 + 15x - 25 -> x^2 - (3/10)x + (1/2)`.
    /// ```
    /// # use num::Rational32;
    /// # use adic::{Polynomial, EAdic};
    /// let poly = Polynomial::<EAdic>::new_with_prime(5, vec![-25, 15, -50]);
    /// let expected = vec![Rational32::new(1, 2), Rational32::new(-3, 10), Rational32::new(1, 1)];
    /// assert_eq!(Polynomial::new_with_prime(5, expected), poly.monic());
    /// ```
    pub fn monic(self) -> Polynomial<QAdic<A>> {

        let leading_coefficient = self.leading_coefficient();
        let mut new_coeffs = match leading_coefficient {
            Some(lc) => self.into_coefficients().map(
                |c| QAdic::from_integer(c) / QAdic::from_integer(lc.clone())
            ).collect::<Vec<_>>(),
            _ => vec![],
        };

        // Set the leading coefficient explicitly with norm and a unit of "one"
        if let Some(mc) = new_coeffs.last_mut() {
            *mc = mc.local_one();
        }
        Polynomial::new(new_coeffs)

    }

}


impl<A> Polynomial<A>
where A: AdicInteger + Into<ZAdic> {

    /// Solve for the roots of `Polynomial` and return the [`Variety`](crate::Variety)
    ///
    /// The algorithm works in two steps:
    /// - Find all solutions mod p^(2*v+1), where v is the valuation of f'(x) at each solution
    /// - Hensel lift to calculate the remaining digits
    ///
    /// Hensel's lemma basically says if the valuation of f(x) is more than twice that of f'(x),
    ///  then you are close to a simple root of the polynomial
    ///  and you can "lift" this approximate root of the polynomial to a full p-adic root.
    /// That is why there are two steps: first find an approximate root and then lift it to a true root.
    ///
    /// In the simplest case, you find solutions in F_p (solutions mod p) and "lift" those to F_p^2, F_p^3, etc.
    /// With the Newton-ish approximation of `f(y) = f(x) + f'(y-x) (y-x)`, you plug in each new digit:
    /// - `f(r_k) = f(r_{k-1} + d p^k) = f(r_{k-1}) + d p^k f'(r_{k-1}) mod p^{k+1}`
    /// - `f(r_{k-1}) = F p^k = 0 mod p^k`
    /// - `0 = f(r_k) = f(r_{k-1}) + d * p^k f'(r) = p^k (F + d * f'(r)) mod p^{k+1}`
    /// - `d = - F * (f'(r))^{-1} mod p = - (p^{-k} f(r_{k-1})) (f'(r_{k-1}))^{-1} mod p`
    ///
    /// This gives the next digit d of the root from the last guess, `r_{k-1}`.
    /// If n has a factor of p, then the algorithm is more complicated because you have to take into account more digits.
    ///
    /// In the case that f(x) NEVER has valuation more than twice that of f'(x), Hensel's lemma fails.
    /// But in this case, the derivative also approaches zero, meaning this is actually a double root.
    /// We find degenerate roots by performing
    ///  [square-free decomposition](<https://en.wikipedia.org/wiki/Square-free_polynomial#Yun's_algorithm>) of the `Polynomial` first.
    ///
    /// 5-adic (x + 1)(x + 2) = 1 + 3 x + x^2
    /// ```
    /// # use adic::{EAdic, Polynomial, QAdic, Variety, ZAdic};
    /// let p = 5;
    /// let f = Polynomial::<EAdic>::new_with_prime(p, vec![2, 3, 1]);
    /// let precision = 6;
    /// let expected = Variety::new(vec![
    ///     QAdic::from_integer(ZAdic::new_approx(p, 6, vec![3, 4, 4, 4, 4, 4])),
    ///     QAdic::from_integer(ZAdic::new_approx(p, 6, vec![4, 4, 4, 4, 4, 4])),
    /// ]);
    /// let variety = f.variety(precision)?;
    /// assert_eq!(expected, variety);
    /// assert_eq!("variety(...444443._5, ...444444._5)", variety.to_string());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// 7-adic (x + 1)(x^2 - 2) = - 2 - 2 x + x^2 + x^3
    /// ```
    /// # use adic::{EAdic, Polynomial, QAdic, Variety, ZAdic};
    /// let p = 7;
    /// let f = Polynomial::<EAdic>::new_with_prime(p, vec![-2, -2, 1, 1]);
    /// let expected = Variety::new(vec![
    ///     QAdic::from_integer(ZAdic::new_approx(p, 6, vec![3, 1, 2, 6, 1, 2])),
    ///     QAdic::from_integer(ZAdic::new_approx(p, 6, vec![4, 5, 4, 0, 5, 4])),
    ///     QAdic::from_integer(ZAdic::new_approx(p, 6, vec![6, 6, 6, 6, 6, 6])),
    /// ]);
    /// let variety = f.variety(6)?;
    /// assert_eq!(expected, variety);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// `Polynomial`'s `certainty` is not high enough for desired `precision`
    ///
    /// # Panics
    /// Panics if certainty does not behave as expected
    pub fn variety(&self, precision: isize) -> AdicResult<Variety<QAdic<ZAdic>>> {
        let qadic_poly = Polynomial::<QAdic<ZAdic>>::new(
            self.coefficients().cloned().map(|a| QAdic::from_integer(a.into())).collect()
        );
        qadic_polynomial_variety(qadic_poly, precision)
    }

    /// Return the size of the variety for this `Polynomial`: the number of simple roots
    ///
    /// 7-adic `(x + 1)(x^2 - 2)(x^2 - 3) = 6 + 6 x - 5 x^2 - 5 x^3 + x^4 + x^5`
    ///
    /// Expect size 3 because -1 and +/- sqrt(2) exist in 7-adics but not +/- sqrt(3).
    /// ```
    /// # use adic::{EAdic, Polynomial};
    /// let f = Polynomial::<EAdic>::new_with_prime(7, vec![6, 6, -5, -5, 1, 1]);
    /// assert_eq!(Ok(3), f.variety_size());
    /// ```
    ///
    /// # Errors
    /// Errors if rootfinding encounters problems
    pub fn variety_size(&self) -> AdicResult<usize> {
        let qadic_poly = Polynomial::<QAdic<ZAdic>>::new(
            self.coefficients().cloned().map(|a| QAdic::from_integer(a.into())).collect()
        );
        qadic_polynomial_variety_size(qadic_poly)
    }

    #[must_use]
    /// Greatest common divisor between `ZAdic` polynomials
    ///
    /// Note: Use this function rather than
    ///  [`gcd`](Polynomial::gcd) or [`primitive_gcd`](Polynomial::primitive_gcd)!
    /// `ZAdic` is different than others in how it can have approximate zeros and needs to be
    ///  wrapped before using this algorithm.
    pub fn zadic_gcd(&self, other: &Self) -> Polynomial<ZAdic> {
        let za = Polynomial::new(self.coefficients().map(|c| c.clone().into()).collect());
        let zb = Polynomial::new(other.coefficients().map(|c| c.clone().into()).collect());
        let a = zadic_wrapper::wrap_poly(za);
        let b = zadic_wrapper::wrap_poly(zb);
        let gcd = euclid::primitive_pseudo_rem_gcd(a, b);
        Polynomial::new(gcd.into_coefficients().map(|z| z.0).collect())
    }

    #[must_use]
    /// Split `Polynomial<ZAdic>` into primitive-monic square-free polynomials to successive powers
    ///
    /// Note: Use this function rather than
    ///  [`raw_square_free_decomposition`](Polynomial::raw_square_free_decomposition) or
    ///  [`primitive_square_free_decomposition`](Polynomial::primitive_square_free_decomposition)!
    /// `ZAdic` is different than others in how it can have approximate zeros and needs to be
    ///  wrapped before using this algorithm.
    ///
    /// `f(x) = a_1 a_2^2 a_3^3 ... a_d^d`
    /// `f.square_free_decomposition() ~> vec![a_1, a_2, a_3, ... a_d]` (up to a constant multiple)
    pub fn zadic_square_free_decomposition(&self) -> Vec<Polynomial<ZAdic>> {
        let zadic_poly = Polynomial::new(self.coefficients().cloned().map(|a| Into::<ZAdic>::into(a)).collect());
        factorization::zadic_square_free_decomposition(&zadic_poly)
    }

}


impl<A> Polynomial<QAdic<A>>
where A: AdicInteger, QAdic<A>: Into<QAdic<ZAdic>> {

    /// Solve for the roots of `Polynomial` and return the [`Variety`](crate::Variety)
    ///
    /// This works just like [`Polynomial::<A>::variety`](crate::Polynomial::variety) but for adic numbers, i.e. [`QAdic`].
    ///
    /// `(x - 1/7)^2 (x^2 - 2) (x^2 - 3) -> (1/5, 1/5, +sqrt(2), -sqrt(2))`
    ///
    /// ```
    /// # use num::Rational32;
    /// # use adic::{EAdic, Polynomial, QAdic, Variety, ZAdic};
    /// let p = 7;
    /// let f = Polynomial::<QAdic<EAdic>>::new_with_prime(p, vec![
    ///     Rational32::new(6, 49), Rational32::new(-12, 7), Rational32::new(289, 49), Rational32::new(10, 7),
    ///     Rational32::new(-244, 49), Rational32::new(-2, 7), Rational32::new(1, 1),
    /// ]);
    /// let expected = Variety::new(vec![
    ///     QAdic::new(ZAdic::new_approx(p, 6, vec![3, 1, 2, 6, 1, 2]), 0),
    ///     QAdic::new(ZAdic::new_approx(p, 6, vec![4, 5, 4, 0, 5, 4]), 0),
    ///     QAdic::new(ZAdic::new_approx(p, 7, vec![1]), -1),
    ///     QAdic::new(ZAdic::new_approx(p, 7, vec![1]), -1),
    /// ]);
    /// let variety = f.variety(6)?;
    /// assert_eq!(expected, variety);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn variety(&self, precision: isize) -> AdicResult<Variety<QAdic<ZAdic>>> {
        let qadic_poly = Polynomial::<QAdic<ZAdic>>::new(
            self.coefficients().cloned().map(QAdic::<A>::into).collect()
        );
        qadic_polynomial_variety(qadic_poly, precision)
    }

    /// Return the size of the variety for this `Polynomial`: the number of simple roots
    ///
    /// This works just like [`Polynomial::<A>::variety_size`](crate::Polynomial::variety_size) but for adic numbers, i.e. [`QAdic`].
    pub fn variety_size(&self) -> AdicResult<usize> {
        let qadic_poly = Polynomial::<QAdic<ZAdic>>::new(
            self.coefficients().cloned().map(QAdic::<A>::into).collect()
        );
        qadic_polynomial_variety_size(qadic_poly)
    }

}


impl<A, N> PrimedFrom<Polynomial<N>> for Polynomial<A>
where A: AdicPrimitive, N: Clone + LocalZero + PrimedInto<A> {
    fn primed_from<P>(p: P, n: Polynomial<N>) -> Self
    where P: Into<Prime> {
        let p = p.into();
        Self::new_with_prime(p, n.into_coefficients().collect())
    }
}
