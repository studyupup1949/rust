use std::{
    iter::{once, repeat},
    ops::Sub,
};
use num::{BigInt, Integer, One};
use crate::{
    error::{validate_matching_p, AdicError, AdicResult},
    combinatorics::carmichael,
    divisible::Prime,
    normed::{UltraNormed, Valuation, ValuationRing},
    traits::{AdicInteger, AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits, HasDigits, PrimedFrom},
};
use super::{EAdic, QAdic, RAdic, ZAdic};



#[derive(Debug, Clone)]
/// Lazy struct holding onto adic number division for further information,
///  e.g. either exact or approximate division and how much precision.
/// It handles both integer division with [`zapprox`](Self::zapprox)/[`zapprox_max`](Self::zapprox_max) (this will TRUNCATE)
///  and fractional division with [`qapprox`](Self::qapprox)/[`qapprox_max`](Self::qapprox_max).
pub (crate) struct LazyDiv<A>
where A: AdicPrimitive {
    a: A,
    b: A,
}

impl<A> LazyDiv<A>
where A: AdicPrimitive {

    /// Create a new `LazyDiv`. Probably should only be used from `AdicPrimitive` div methods.
    pub fn new(a: A, b: A) -> Self {
        validate_matching_p([a.p(), b.p()]);
        Self {
            a,
            b,
        }
    }

    /// Prime for `LazyDiv`; validated to match `a` and `b` on creation
    pub fn p(&self) -> Prime {
        self.a.p()
    }

}

impl<A, AU, VR, E> LazyDiv<A>
where
A: AdicPrimitive
    + UltraNormed<ValuationRing = VR, Unit = AU>,
AU: AdicInteger + Into<EAdic>,
VR: ValuationRing + Sub<Output = VR>,
isize: TryFrom<VR, Error=E>,
AdicError: From<E> {

    /// Perform the exact integer division, giving a `EAdic`.
    /// Note the input must be convertible to `EAdic`.
    /// E.g. approximate numbers cannot use this function.
    ///
    /// # Errors
    /// Returns an error if the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{error::AdicError, traits::{AdicInteger, AdicPrimitive, PrimedFrom}, EAdic, QAdic};
    /// let (neg_one, four, five) = (EAdic::new_neg(5, vec![]), EAdic::new(5, vec![4]), EAdic::new(5, vec![0, 1]));
    /// assert_eq!(EAdic::new_repeating(5, vec![], vec![1]), neg_one.clone() / four.clone());
    /// assert_eq!(EAdic::new_neg(5, vec![1]), four.clone() / neg_one.clone());
    /// assert_eq!(neg_one, neg_one.clone() / five.clone());
    /// assert_eq!(EAdic::zero(5), four.clone() / five.clone());
    /// let four = EAdic::primed_from(5, 4);
    /// let neg_one = EAdic::primed_from(5, -1);
    /// let neg_four = EAdic::primed_from(5, -4);
    /// let neg_one_fourth = EAdic::new_repeating(5, vec![], vec![1]);
    /// assert_eq!(neg_one_fourth, neg_one.clone() / four.clone());
    /// assert_eq!(neg_four, four.clone() / neg_one.clone());
    /// // let bad_div = neg_one.clone() / EAdic::zero(5);
    /// ```
    pub fn int_div_exact(self) -> AdicResult<EAdic> {
        self.div_exact().map(|q| q.frac_and_int().1)
    }

    /// Perform the exact integer remainder, giving a `UAdic`.
    /// Note the input must be convertible to `EAdic`.
    /// E.g. approximate numbers cannot use this function.
    ///
    /// # Errors
    /// Returns an error if the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{error::AdicError, traits::{AdicInteger, AdicPrimitive, PrimedFrom}, EAdic, QAdic};
    /// let (neg_one, four, five) = (EAdic::new_neg(5, vec![]), EAdic::new(5, vec![4]), EAdic::new(5, vec![0, 1]));
    /// assert_eq!(EAdic::zero(5), neg_one.clone() % four.clone());
    /// assert_eq!(EAdic::zero(5), four.clone() % neg_one.clone());
    /// assert_eq!(four, neg_one.clone() % five.clone());
    /// assert_eq!(four, four.clone() % five.clone());
    /// // let bad_rem = neg_one.clone() % EAdic::zero(5);
    /// ```
    pub fn rem_exact(self) -> AdicResult<EAdic> {
        match (self.a.valuation(), self.b.valuation()) {
            (_, Valuation::PosInf) => Err(AdicError::DivideByZero),
            (Valuation::PosInf, Valuation::Finite(_)) => Ok(EAdic::zero(self.p())),
            (Valuation::Finite(va), Valuation::Finite(vb)) if va > vb => Ok(EAdic::zero(self.p())),
            (Valuation::Finite(va), Valuation::Finite(vb)) => self.div_exact().and_then(|q| {
                let frac = q.frac_and_int().0;
                let vdiff = (vb - va).try_into_u32()?;
                Ok(EAdic::from((frac * QAdic::from_prime_power((5, vdiff))).try_into_integer()?))
            })
        }
    }

    /// Perform the exact division, giving a `QAdic<RAdic>`.
    /// Note the input must be convertible to `QAdic<RAdic>`.
    /// E.g. approximate numbers cannot use this function.
    ///
    /// # Errors
    /// Returns an error if the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{error::AdicError, traits::{AdicPrimitive, PrimedFrom}, EAdic, QAdic};
    /// let four = QAdic::primed_from(5, 4);
    /// let five = QAdic::primed_from(5, 5);
    /// let neg_one = QAdic::primed_from(5, -1);
    /// let neg_four = QAdic::primed_from(5, -4);
    /// let neg_one_fourth = QAdic::primed_from(5, num::Rational32::new(-1, 4));
    /// assert_eq!(neg_one_fourth, neg_one.clone() / four.clone());
    /// assert_eq!(neg_four, four.clone() / neg_one.clone());
    /// assert_eq!(QAdic::primed_from(5, num::Rational32::new(-1, 5)), neg_one.clone() / five.clone());
    /// assert_eq!(QAdic::primed_from(5, num::Rational32::new(4, 5)), four.clone() / five.clone());
    /// assert_eq!(25 * neg_one_fourth.clone(), neg_one.clone() / QAdic::new(EAdic::primed_from(5, 4), -2));
    /// assert_eq!(25 * neg_one_fourth.clone(), QAdic::new(EAdic::primed_from(5, -1), 2) / four.clone());
    /// // let bad_div = neg_one.clone() / QAdic::zero(5);
    /// ```
    pub fn div_exact(self) -> AdicResult<QAdic<EAdic>> {

        let p = self.a.p();
        let (Some(ub), Valuation::Finite(vb)) = self.b.into_unit_and_valuation() else {
            return Err(AdicError::DivideByZero)
        };
        let vb = isize::try_from(vb)?;
        let (Some(ua), Valuation::Finite(va)) = self.a.into_unit_and_valuation() else {
            return Ok(QAdic::zero(p));
        };
        let va = isize::try_from(va)?;

        let ra = Into::<EAdic>::into(ua).into();
        let rb = Into::<EAdic>::into(ub).into();
        let div = div_by_unit_r(&ra, &rb)?;
        Ok(QAdic::new(div.into(), va - vb))

    }

}


impl<A, AU, VR, E> LazyDiv<A>
where
A: AdicPrimitive
    + HasApproximateDigits<DigitIndex = VR>
    + UltraNormed<ValuationRing = VR, Unit = AU>,
AU: CanApproximate<Approximation = ZAdic> + AdicInteger,
VR: ValuationRing + Sub<Output = VR>,
isize: TryFrom<VR, Error=E>,
AdicError: From<E> {

    #[allow(unused)]
    /// Perform the approximate integer division with given precision, giving a `ZAdic`.
    /// Note the input should have a `significance` of at least `precision` to succeed.
    /// E.g. dividing by `ZAdic::new_approx(5, 4, vec![0, 0, 0, 0])` will always return an error
    ///  (unless `a` is zero) because it is LIKE dividing by zero.
    ///
    /// Note: unused since `LazyDiv` is no longer exposed to the api.
    ///
    /// # Errors
    /// Returns an error if the numbers are not precise enough or the denominator is zero or integer conversion failure
    pub fn int_div_approx(self, precision: usize) -> AdicResult<ZAdic> {
        let iprecision = <isize as TryFrom<usize>>::try_from(precision)?;
        self.div_approx(iprecision).map(|q| q.frac_and_int().1)
    }

    #[allow(unused)]
    pub fn rem_approx(self, precision: usize) -> AdicResult<ZAdic> {
        let p = self.p();
        let iprecision = <isize as TryFrom<usize>>::try_from(precision)?;
        match (self.a.valuation(), self.b.valuation()) {
            (_, Valuation::PosInf) => Err(AdicError::DivideByZero),
            (Valuation::PosInf, Valuation::Finite(_)) => Ok(ZAdic::zero(p)),
            (Valuation::Finite(va), Valuation::Finite(vb)) if va > vb => Ok(ZAdic::zero(p)),
            (Valuation::Finite(va), Valuation::Finite(vb)) => self.div_approx(iprecision).and_then(|q| {
                let frac = q.frac_and_int().0;
                let vdiff = (vb - va).try_into_u32()?;
                let integer = (frac * QAdic::from_prime_power((p, vdiff))).try_into_integer()?;
                if iprecision >= 0 {
                    Ok(ZAdic::from(integer))
                } else if iprecision + isize::try_from(vb - va)? >= 0 {
                    let rem_cert = usize::try_from(iprecision + isize::try_from(vb - va)?)?;
                    Ok(integer.into_approximation(rem_cert))
                } else {
                    Ok(ZAdic::empty(p))
                }
            })
        }
    }

    /// Perform the approximate adic integer division with maximum possible precision, giving a `ZAdic`.
    /// This will take the input's significance into account and propagate it.
    ///
    /// # Errors
    /// Returns an error if the input has infinite precision or the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{error::AdicError, traits::{AdicPrimitive, CanApproximate, PrimedFrom}, EAdic, QAdic, ZAdic};
    /// let four = ZAdic::primed_from(5, 4);
    /// let five = ZAdic::primed_from(5, 5);
    /// let neg_one = ZAdic::primed_from(5, -1);
    /// let neg_one_fourth = ZAdic::from(EAdic::new_repeating(5, vec![], vec![1]));
    /// assert_eq!(neg_one_fourth.approximation(2), neg_one.clone() / four.approximation(2));
    /// assert_eq!(neg_one_fourth.approximation(2), neg_one.approximation(3) / four.approximation(2));
    /// assert_eq!(neg_one_fourth.approximation(4), neg_one.clone() / four.approximation(4));
    /// assert_eq!(neg_one_fourth.approximation(3), neg_one.approximation(3) / four.approximation(4));
    /// assert_eq!(neg_one.approximation(1), neg_one.clone() / five.approximation(3));
    /// assert_eq!(ZAdic::zero(5).approximation(1), four.clone().approximation(2) / five.clone());
    /// // let bad_div = neg_one.clone() / ZAdic::zero(5).approximation(4);
    /// ```
    pub fn int_div_approx_max(self) -> AdicResult<ZAdic> {
        self.div_approx_max().map(|q| q.frac_and_int().1)
    }

    /// Perform approximate adic integer remainder with maximum possible precision.
    ///
    /// # Errors
    /// Returns an error if the input has infinite precision or the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{error::AdicError, traits::{AdicPrimitive, CanApproximate, PrimedFrom}, EAdic, QAdic, ZAdic};
    /// let four = ZAdic::primed_from(5, 4);
    /// let five = ZAdic::primed_from(5, 5);
    /// let neg_one = ZAdic::primed_from(5, -1);
    /// let neg_one_fourth = ZAdic::from(EAdic::new_repeating(5, vec![], vec![1]));
    /// assert_eq!(ZAdic::zero(5), neg_one.clone() % four.approximation(2));
    /// assert_eq!(ZAdic::zero(5), neg_one.approximation(3) % four.approximation(2));
    /// assert_eq!(ZAdic::zero(5), neg_one.clone() % four.approximation(4));
    /// assert_eq!(ZAdic::zero(5), neg_one.approximation(3) % four.approximation(4));
    /// assert_eq!(four, neg_one.clone() % five.approximation(3));
    /// assert_eq!(four, four.clone().approximation(2) % five.clone());
    /// // let bad_div = neg_one.clone() % ZAdic::zero(5).approximation(4);
    /// ```
    pub fn rem_approx_max(self) -> AdicResult<ZAdic> {

        let p = self.p();
        match (self.a.valuation(), self.b.valuation()) {
            (_, Valuation::PosInf) => Err(AdicError::DivideByZero),
            (Valuation::PosInf, Valuation::Finite(_)) => Ok(ZAdic::zero(self.p())),
            (Valuation::Finite(va), Valuation::Finite(vb)) if va > vb => Ok(ZAdic::zero(self.p())),
            (Valuation::Finite(va), Valuation::Finite(vb)) => {

                let max_prec = self.max_precision()?;
                self.div_approx_max().and_then(|q| {

                    let frac = q.frac_and_int().0;
                    let vdiff = (vb - va).try_into_u32()?;
                    let integer = (frac * QAdic::from_prime_power((p, vdiff))).try_into_integer()?;
                    // If max precision is less than zero valuation, the remainder may be approximate
                    match max_prec {
                        Valuation::PosInf => Ok(ZAdic::from(integer)),
                        Valuation::Finite(iprecision) if iprecision >= 0 => Ok(ZAdic::from(integer)),
                        Valuation::Finite(iprecision) if iprecision + isize::try_from(vb - va)? >= 0 => {
                            let rem_cert = usize::try_from(iprecision + isize::try_from(vb - va)?)?;
                            Ok(integer.into_approximation(rem_cert))
                        },
                        Valuation::Finite(_) => Ok(ZAdic::empty(p)),
                    }

                })

            },
        }
    }

    /// Perform the approximate division with given precision, giving a `QAdic<ZAdic>`.
    /// Note the input should have a `significance` of at least `precision` to succeed.
    /// E.g. dividing by `QAdic::new(ZAdic::new_approx(5, 4, vec![0, 0, 0, 0]), 0)` will always return an error
    ///  (unless `a` is zero) because it is LIKE dividing by zero.
    ///
    /// Note: used indirectly since `LazyDiv` is no longer exposed to the api.
    ///
    /// # Errors
    /// Returns an error if the numbers are not precise enough or the denominator is zero or integer conversion failure
    pub fn div_approx(self, precision: isize) -> AdicResult<QAdic<ZAdic>> {

        let p = self.a.p();
        let ca = self.a.certainty().convert::<isize>()?;
        let cb = self.b.certainty().convert::<isize>()?;

        let (ub, vb) = match self.b.into_unit_and_valuation() {
            (Some(u), Valuation::Finite(v)) => (u, v),
            (None, Valuation::Finite(_)) => {
                return Ok(QAdic::empty(p, precision));
            },
            _ => {
                return Err(AdicError::DivideByZero)
            },
        };
        let vb = isize::try_from(vb)?;
        let (Some(ua), Valuation::Finite(va)) = self.a.into_unit_and_valuation() else {
            return Ok(QAdic::empty(p, precision));
        };
        let va = isize::try_from(va)?;

        let adjusted_precision = precision + vb - va;
        if adjusted_precision <= 0 {
            Ok(QAdic::empty(p, precision))
        } else if ca < (va + adjusted_precision).into() || cb < (vb + adjusted_precision).into() {
            Err(AdicError::InappropriatePrecision(
                format!("a and b not precise enough to give {precision} digits")
            ))
        } else {
            let ap = adjusted_precision.unsigned_abs();
            let div = ua.into_approximation(ap) * invert_unit_z(&ub.into_approximation(ap))?;
            Ok(QAdic::new(div, va - vb))
        }

    }

    /// Perform the approximate division with maximum possible precision, giving a `QAdic<ZAdic>`.
    /// This will take the input's significance into account and propagate it.
    ///
    /// # Errors
    /// Returns an error if the input has infinite precision or the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{error::AdicError, traits::{AdicPrimitive, CanApproximate, PrimedFrom}, EAdic, QAdic, ZAdic};
    /// let four: QAdic<ZAdic> = QAdic::primed_from(5, 4);
    /// let five: QAdic<ZAdic> = QAdic::primed_from(5, 5);
    /// let neg_one: QAdic<ZAdic> = QAdic::primed_from(5, -1);
    /// let neg_four: QAdic<ZAdic> = QAdic::primed_from(5, -4);
    /// let neg_one_fourth: QAdic<ZAdic> = QAdic::primed_from(5, num::Rational32::new(-1, 4));
    /// assert_eq!(neg_one_fourth.approximation(2), neg_one.clone() / four.approximation(2));
    /// assert_eq!(neg_one_fourth.approximation(2), neg_one.approximation(3) / four.approximation(2));
    /// assert_eq!(neg_one_fourth.approximation(4), neg_one.clone() / four.approximation(4));
    /// assert_eq!(neg_one_fourth.approximation(3), neg_one.approximation(3) / four.approximation(4));
    /// assert_eq!((neg_one.clone() / five.clone()).approximation(1), neg_one.clone() / five.approximation(3));
    /// assert_eq!((four.clone() / five.clone()).approximation(1), four.clone().approximation(2) / five.clone());
    ///
    /// let neg_twenty_five = 25 * neg_one.clone();
    /// let four_twenty_fifth = QAdic::new(ZAdic::primed_from(5, 4), -2);
    /// let neg_twenty_five_fourth = 25 * neg_one_fourth.clone();
    /// let approx_div = neg_one.approximation(3) / four_twenty_fifth.approximation(4);
    /// assert_eq!(neg_twenty_five_fourth.approximation(5), approx_div);
    /// let approx_div = neg_twenty_five.approximation(3) / four.approximation(4);
    /// assert_eq!(neg_twenty_five_fourth.approximation(3), approx_div);
    /// // let bad_div = neg_one.clone() / QAdic::zero(5).approximation(4);
    /// ```
    pub fn div_approx_max(self) -> AdicResult<QAdic<ZAdic>> {

        let p = self.a.p();
        match (self.a.valuation(), self.b.valuation()) {
            (Valuation::Finite(_), Valuation::Finite(_)) => {
                match self.max_precision()? {
                    Valuation::Finite(prec) => self.div_approx(prec),
                    Valuation::PosInf => Err(AdicError::InappropriatePrecision(
                        "Cannot approximate an infinite division".to_string()
                    )),
                }
            },
            (Valuation::PosInf, Valuation::Finite(_)) => {
                Ok(QAdic::zero(p))
            },
            (_, Valuation::PosInf) => {
                Err(AdicError::DivideByZero)
            },
        }

    }

    fn max_precision(&self) -> AdicResult<Valuation<isize>> {

        match (self.a.valuation(), self.b.valuation()) {
            (Valuation::Finite(val_a), Valuation::Finite(val_b)) => {

                let va = isize::try_from(val_a)?;
                let vb = isize::try_from(val_b)?;
                let ca = self.a.certainty().convert::<isize>()?;
                let cb = self.b.certainty().convert::<isize>()?;
                match (ca, cb) {
                    (Valuation::Finite(ca), Valuation::Finite(cb)) => Ok(
                        Valuation::Finite(std::cmp::min(ca - vb, cb + va - 2*vb))
                    ),
                    (Valuation::Finite(ca), _) => Ok(Valuation::Finite(ca - vb)),
                    (_, Valuation::Finite(cb)) => Ok(Valuation::Finite(cb + va - 2*vb)),
                    _ => Ok(Valuation::PosInf),
                }

            },
            (Valuation::PosInf, Valuation::Finite(_)) => {
                Ok(Valuation::PosInf)
            },
            (_, Valuation::PosInf) => {
                Err(AdicError::DivideByZero)
            },
        }

    }

}



fn div_by_unit_r(a: &RAdic, b: &RAdic) -> AdicResult<RAdic> {

    let p = b.p();

    // Invert unit, negate valuation
    // We take the inverse of the QAdic's UNIT and drop the first VALUATION digits.
    // This essentially skips the fractional part.

    // First, convert to BigRational
    let br = a.big_rational_value() / b.big_rational_value();
    let (numer, denom) = br.into_raw();

    // Next, calculate (almost exact) replen with the Carmichael function
    let replen = if denom == BigInt::one() {
        return Ok(RAdic::primed_from(p, numer));
    } else {
        let mag = denom.magnitude().clone();
        let h = carmichael(mag);
        usize::try_from(h)?
    };

    if replen > 1000 {
        println!("WARNING: the computation of {a} / {b} will take {replen} digits...");
    }

    // Convert to the form a + (-b) / c, where -c < (-b) <= 0, an integer and all-repeating digits
    let int_term: BigInt = (numer.clone() + denom.clone() - BigInt::one()).div_floor(&denom);
    let small_neg_numer: BigInt = numer.clone() - int_term.clone() * denom.clone();

    // Convert back to RAdic with long division

    // Long division
    // ( (2)4._5 ).inv() = (3/2).inv() = 2/3 = 1 - 1/3 = 1. + 44./03.
    //     ___
    // 03 | 44
    // x3  -14
    //      3
    // x1  -3
    // => inv = 1._5 + (13)._5 = (31)4._5

    let mut repeating = vec![];
    let mut divisible = ZAdic::primed_from(
        p,
        i32::try_from(small_neg_numer.clone())?,
    ).into_approximation(replen);
    let mut divisor = ZAdic::primed_from(
        p,
        i32::try_from(denom)?,
    ).into_approximation(replen);
    let Ok(unit_first) = divisor.digit0() else {
        return Err(AdicError::DivideByZero);
    };
    let first_inv = p.mod_inv(unit_first);

    // WARNING: The following seems inefficient, with the divisble subtraction
    while let Ok(first_digit) = divisible.digit0() {
        let d = first_digit * first_inv % p;
        repeating.push(d);
        divisible = divisible - d * divisor.clone();
        divisible = divisible.into_quotient(1);
        divisor.pop_digit()?;
    }

    // Now put it all together
    let int_adic = RAdic::primed_from(
        p,
        i32::try_from(int_term)?,
    );
    let repeat_adic = RAdic::new(p, vec![], repeating);
    Ok(int_adic + repeat_adic)

}


fn invert_unit_z(z: &ZAdic) -> AdicResult<ZAdic> {

    let p = z.p();
    let Valuation::Finite(significance) = z.significance() else {
        return Err(AdicError::InappropriatePrecision("cannot take inverse of an exact ZAdic".to_string()));
    };

    // If no first digit, it's empty, and the inverse of "unit empty" is empty
    let Ok(unit_first) = z.digit0() else {
        return Ok(ZAdic::empty(p));
    };

    // Invert unit, negate valuation

    // Long division
    // (...0023._5).inv()
    //       _____
    // 0023 | 0001
    //   x2  -0101
    //        4400
    //   x0  -000
    //        4400
    //   x3  -24
    //        2000
    //   x4  -2
    // => inv = ...4302

    let mut inverse = vec![];
    let first_inv = p.mod_inv(unit_first);
    let mut numer = once(1).chain(repeat(0)).take(significance).collect::<Vec<_>>();
    let neg_divisor = (-z.clone()).digits().take(significance).collect::<Vec<_>>();

    // Performance critical!
    // If we can find a more performant inversion algorithm, that would be great.
    for num_done in 0..significance {
        let first_digit = numer[num_done];
        let d = first_digit * first_inv % p;
        inverse.push(d);
        let mut carry = 0;
        for (idx, digit) in numer[num_done..significance].iter_mut().enumerate() {
            let new_digit = *digit + d * neg_divisor[idx] + carry;
            carry = new_digit / p;
            *digit = new_digit % p;
        }
    }

    // Inverse unit with negative valuation
    Ok(ZAdic::new_approx(p, significance, inverse))

}
