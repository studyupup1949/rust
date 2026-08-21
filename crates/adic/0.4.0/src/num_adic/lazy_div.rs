use std::{
    iter::{once, repeat},
    ops::Sub,
};
use crate::{
    adic_valid, special_function::carmichael,
    AdicError, AdicResult,
};
use num::{BigInt, One, Signed};
use super::{
    AdicApproximate, AdicFraction, AdicInteger, AdicNumber, AdicSized, AdicValuation, AdicValuationRing,
    HasDigits, QAdic, RAdic, SignedAdicNumber, ZAdic
};



#[derive(Debug, Clone)]
/// Lazy struct holding onto adic number division for further information,
///  e.g. either exact or approximate division and how much precision.
/// It handles both integer division with [`zapprox`](Self::zapprox)/[`zapprox_max`](Self::zapprox_max) (this will TRUNCATE)
///  and fractional division with [`qapprox`](Self::qapprox)/[`qapprox_max`](Self::qapprox_max).
pub struct LazyDiv<A>
where A: AdicNumber {
    a: A,
    b: A,
}

impl<A> LazyDiv<A>
where A: AdicNumber {
    /// Create a new `LazyDiv`. Probably should only be created from `AdicNumber` div methods.
    pub fn new(a: A, b: A) -> Self {
        adic_valid::validate_mono_character(a.p(), b.p());
        Self {
            a,
            b,
        }
    }
}

impl<A, AU, VR> LazyDiv<A>
where
A: AdicNumber
    + AdicApproximate<DigitIndex = VR>
    + AdicSized<ValuationRing = VR, AdicUnit = AU>,
AU: AdicApproximate + AdicInteger,
VR: AdicValuationRing + Sub<Output = VR> {

    /// Perform the exact integer division, giving a `RAdic`.
    /// Note the input must be convertible to `RAdic`.
    /// E.g. approximate numbers cannot use this function.
    ///
    /// # Errors
    /// Returns an error if the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{iadic_neg, iadic_pos, qadic, radic, AdicError, AdicInteger, AdicNumber, IAdic};
    /// let (neg_one, four) = (iadic_neg!(5, []), iadic_pos!(5, [4]));
    /// let intermediate_div = &neg_one / &four;
    /// assert_eq!(Ok(radic!(5, [], [1])), intermediate_div.rexact());
    /// let bad_div = &neg_one / IAdic::zero(5);
    /// assert_eq!(Err(AdicError::DivideByZero), bad_div.rexact());
    /// let fractional_div = qadic!(neg_one.clone(), 1) / qadic!(four.clone(), -1);
    /// assert_eq!(Ok(radic!(5, [0, 0], [1])), fractional_div.rexact());
    /// let fractional_div = qadic!(neg_one.clone(), -1) / qadic!(four.clone(), 1);
    /// assert_eq!(Ok(radic!(5, [], [1])), fractional_div.rexact());
    /// ```
    pub fn rexact<E>(self) -> AdicResult<RAdic>
    where AU: Into<RAdic>, isize: TryFrom<VR, Error=E>, AdicError: From<E> {
        self.qexact().map(|q| q.frac_and_int().1)
    }

    /// Perform the approximate integer division with given precision, giving a `ZAdic`.
    /// Note the input should have a `significance` of at least `precision` to succeed.
    /// E.g. dividing by `zadic_approx!(5, 4, [0, 0, 0, 0])` will always return an error
    ///  (unless `a` is zero) because it is LIKE dividing by zero.
    ///
    /// # Errors
    /// Returns an error if the numbers are not precise enough or the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{iadic_neg, iadic_pos, qadic, zadic_approx, AdicError, AdicInteger, AdicNumber, IAdic};
    /// let (neg_one, four) = (iadic_neg!(5, []), iadic_pos!(5, [4]));
    /// let intermediate_div = &neg_one / &four;
    /// assert_eq!(Ok(zadic_approx!(5, 6, [1, 1, 1, 1, 1, 1])), intermediate_div.zapprox(6));
    /// let bad_div = &neg_one / IAdic::zero(5);
    /// assert_eq!(Err(AdicError::DivideByZero), bad_div.zapprox(6));
    /// let fractional_div = qadic!(neg_one.clone(), 1) / qadic!(four.clone(), -1);
    /// assert_eq!(Ok(zadic_approx!(5, 6, [0, 0, 1, 1, 1, 1])), fractional_div.zapprox(6));
    /// let fractional_div = qadic!(neg_one.clone(), -1) / qadic!(four.clone(), 1);
    /// assert_eq!(Ok(zadic_approx!(5, 6, [1, 1, 1, 1, 1, 1])), fractional_div.zapprox(6));
    /// ```
    pub fn zapprox<E>(self, precision: usize) -> AdicResult<ZAdic>
    where isize: TryFrom<VR, Error=E>, AdicError: From<E> {
        let iprecision = <isize as TryFrom<usize>>::try_from(precision)?;
        self.qapprox(iprecision).map(|q| q.frac_and_int().1)
    }

    /// Perform the approximate integer division with maximum possible precision, giving a `ZAdic`.
    /// This will take the input's significance into account and propagate it.
    ///
    /// # Errors
    /// Returns an error if the input has infinite precision or the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{qadic, zadic_approx, zadic_exact_neg, zadic_exact_pos, AdicError, AdicInteger, AdicFraction, AdicNumber, ZAdic};
    /// let four = zadic_exact_pos!(5, [4]);
    /// let neg_one = zadic_exact_neg!(5, []);
    /// assert_eq!(Ok(zadic_approx!(5, 2, [1, 1])), (&neg_one / four.approximation(2)).zapprox_max());
    /// assert_eq!(Ok(zadic_approx!(5, 2, [1, 1])), (neg_one.approximation(3) / four.approximation(2)).zapprox_max());
    /// assert_eq!(Ok(zadic_approx!(5, 4, [1, 1, 1, 1])), (&neg_one / four.approximation(4)).zapprox_max());
    /// assert_eq!(Ok(zadic_approx!(5, 3, [1, 1, 1])), (neg_one.approximation(3) / four.approximation(4)).zapprox_max());
    /// assert!(matches!((&neg_one / &four).zapprox_max(), Err(AdicError::InappropriatePrecision(_))));
    /// assert_eq!(Err(AdicError::DivideByZero), (&neg_one / ZAdic::zero(5)).zapprox_max());
    /// let fractional_div = qadic!(neg_one.clone(), 1).approximation(3) / qadic!(four.clone(), -1).approximation(4);
    /// assert_eq!(Ok(zadic_approx!(5, 4, [0, 0, 1, 1])), fractional_div.zapprox_max());
    /// let fractional_div = qadic!(neg_one.clone(), -1).approximation(3) / qadic!(four.clone(), 1).approximation(4);
    /// assert_eq!(Ok(zadic_approx!(5, 1, [1])), fractional_div.zapprox_max());
    /// ```
    pub fn zapprox_max<E>(self) -> AdicResult<ZAdic>
    where A: AdicApproximate, isize: TryFrom<VR, Error=E>, AdicError: From<E> {
        self.qapprox_max().map(|q| q.frac_and_int().1)
    }

    /// Perform the exact division, giving a `QAdic<RAdic>`.
    /// Note the input must be convertible to `QAdic<RAdic>`.
    /// E.g. approximate numbers cannot use this function.
    ///
    /// # Errors
    /// Returns an error if the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{iadic_neg, iadic_pos, qadic, radic, AdicError, AdicNumber, IAdic};
    /// let (neg_one, four) = (iadic_neg!(5, []), iadic_pos!(5, [4]));
    /// let intermediate_div = &neg_one / &four;
    /// assert_eq!(Ok(qadic!(radic!(5, [], [1]), 0)), intermediate_div.qexact());
    /// let bad_div = &neg_one / IAdic::zero(5);
    /// assert_eq!(Err(AdicError::DivideByZero), bad_div.qexact());
    /// let fractional_div = qadic!(neg_one.clone(), 1) / qadic!(four.clone(), -1);
    /// assert_eq!(Ok(qadic!(radic!(5, [], [1]), 2)), fractional_div.qexact());
    /// let fractional_div = qadic!(neg_one.clone(), -1) / qadic!(four.clone(), 1);
    /// assert_eq!(Ok(qadic!(radic!(5, [], [1]), -2)), fractional_div.qexact());
    /// ```
    pub fn qexact<E>(self) -> AdicResult<QAdic<RAdic>>
    where AU: Into<RAdic>, isize: TryFrom<VR, Error=E>, AdicError: From<E> {

        let p = self.a.p();
        let (Some(ub), AdicValuation::Finite(vb)) = self.b.into_unit_and_valuation() else {
            return Err(AdicError::DivideByZero)
        };
        let vb = isize::try_from(vb)?;
        let (Some(ua), AdicValuation::Finite(va)) = self.a.into_unit_and_valuation() else {
            return Ok(QAdic::zero(p));
        };
        let va = isize::try_from(va)?;

        let div = ua.into() * invert_unit_r(&ub.into())?;
        Ok(QAdic::new(div, va - vb))

    }

    /// Perform the approximate division with given precision, giving a `QAdic<ZAdic>`.
    /// Note the input should have a `significance` of at least `precision` to succeed.
    /// E.g. dividing by `qadic!(zadic_approx!(5, 4, [0, 0, 0, 0]), 0)` will always return an error
    ///  (unless `a` is zero) because it is LIKE dividing by zero.
    ///
    /// # Errors
    /// Returns an error if the numbers are not precise enough or the denominator is zero or integer conversion failure
    ///
    /// ```
    /// # use adic::{iadic_neg, iadic_pos, qadic, zadic_approx, AdicError, AdicNumber, IAdic};
    /// let (neg_one, four) = (iadic_neg!(5, []), iadic_pos!(5, [4]));
    /// let intermediate_div = &neg_one / &four;
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 6, [1, 1, 1, 1, 1, 1]), 0)), intermediate_div.qapprox(6));
    /// let bad_div = &neg_one / IAdic::zero(5);
    /// assert_eq!(Err(AdicError::DivideByZero), bad_div.qapprox(6));
    /// let fractional_div = qadic!(neg_one.clone(), 1) / qadic!(four.clone(), -1);
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 4, [1, 1, 1, 1]), 2)), fractional_div.qapprox(6));
    /// let fractional_div = qadic!(neg_one.clone(), -1) / qadic!(four.clone(), 1);
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 8, [1, 1, 1, 1, 1, 1, 1, 1]), -2)), fractional_div.qapprox(6));
    /// ```
    pub fn qapprox<E>(self, precision: isize) -> AdicResult<QAdic<ZAdic>>
    where isize: TryFrom<VR, Error=E>, AdicError: From<E> {

        let p = self.a.p();
        let ca = self.a.certainty().convert::<isize, _>()?;
        let cb = self.b.certainty().convert::<isize, _>()?;

        let (Some(ub), AdicValuation::Finite(vb)) = self.b.into_unit_and_valuation() else {
            return Err(AdicError::DivideByZero)
        };
        let vb = isize::try_from(vb)?;
        let (Some(ua), AdicValuation::Finite(va)) = self.a.into_unit_and_valuation() else {
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
    /// # use adic::{qadic, zadic_approx, zadic_exact_neg, zadic_exact_pos, AdicError, AdicFraction, AdicInteger, AdicNumber, ZAdic};
    /// let four = zadic_exact_pos!(5, [4]);
    /// let neg_one = zadic_exact_neg!(5, []);
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 2, [1, 1]), 0)), (&neg_one / four.approximation(2)).qapprox_max());
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 2, [1, 1]), 0)), (neg_one.approximation(3) / four.approximation(2)).qapprox_max());
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 4, [1, 1, 1, 1]), 0)), (&neg_one / four.approximation(4)).qapprox_max());
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 3, [1, 1, 1]), 0)), (neg_one.approximation(3) / four.approximation(4)).qapprox_max());
    /// assert!(matches!((&neg_one / &four).qapprox_max(), Err(AdicError::InappropriatePrecision(_))));
    /// assert_eq!(Err(AdicError::DivideByZero), (&neg_one / ZAdic::zero(5)).qapprox_max());
    /// let fractional_div = qadic!(neg_one.clone(), 1).approximation(3) / qadic!(four.clone(), -1).approximation(4);
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 2, [1, 1]), 2)), fractional_div.qapprox_max());
    /// let fractional_div = qadic!(neg_one.clone(), -1).approximation(3) / qadic!(four.clone(), 1).approximation(4);
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 3, [1, 1, 1]), -2)), fractional_div.qapprox_max());
    /// ```
    pub fn qapprox_max<E>(self) -> AdicResult<QAdic<ZAdic>>
    where A: AdicApproximate, isize: TryFrom<VR, Error=E>, AdicError: From<E> {

        let p = self.a.p();
        match (self.a.valuation(), self.b.valuation()) {
            (AdicValuation::Finite(val_a), AdicValuation::Finite(val_b)) => {

                let va = isize::try_from(val_a)?;
                let vb = isize::try_from(val_b)?;
                let ca = self.a.certainty().convert::<isize, _>()?;
                let cb = self.b.certainty().convert::<isize, _>()?;
                let min_p = match (ca, cb) {
                    (AdicValuation::Finite(ca), AdicValuation::Finite(cb)) => std::cmp::min(ca - vb, cb + va - 2*vb),
                    (AdicValuation::Finite(ca), _) => ca - vb,
                    (_, AdicValuation::Finite(cb)) => cb + va - 2*vb,
                    _ => Err(AdicError::InappropriatePrecision(
                        "Cannot approximate an infinite division; specify a precision with `approx(prec)`".to_string()
                    ))?
                };

                self.qapprox(min_p)

            },
            (AdicValuation::PosInf, AdicValuation::Finite(_)) => {
                Ok(QAdic::zero(p))
            },
            (_, AdicValuation::PosInf) => {
                Err(AdicError::DivideByZero)
            },
        }

    }

}



fn invert_unit_r(r: &RAdic) -> AdicResult<RAdic> {

    let p = r.p();
    let Ok(unit_first) = r.digit(0) else {
        return Err(AdicError::DivideByZero);
    };

    // Invert unit, negate valuation
    // We take the inverse of the QAdic's UNIT and drop the first VALUATION digits.
    // This essentially skips the fractional part.

    // First, convert to BigRational and invert that
    let br = r.big_rational_value();
    let (numer, denom) = br.into_raw();
    // Switch sign; denom should be positive
    let new_numer = if numer.is_positive() { denom } else { - denom };
    let new_denom = numer.abs();

    // Next, calculate (almost exact) replen with the Carmichael function
    let replen = if new_denom == BigInt::one() {
        let numer32 = new_numer.try_into()?;
        return Ok(RAdic::from_i32(p, numer32));
    } else {
        let mag = new_denom.magnitude().clone();
        let h = carmichael(mag);
        usize::try_from(h)?
    };

    if replen > 1000 {
        println!("WARNING: the computation of 1 / {r} will take {replen} digits...");
    }

    // Convert to the form a + (-b) / c, where -c < (-b) <= 0, an integer and all-repeating digits
    let int_term: BigInt = (new_numer.clone() + new_denom.clone() - 1) / new_denom.clone();
    let small_neg_numer: BigInt = new_numer.clone() - int_term.clone() * new_denom.clone();

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
    let mut divisible = ZAdic::from_i32(
        p,
        i32::try_from(small_neg_numer.clone())?,
    ).into_approximation(replen);
    let mut divisor = ZAdic::from_i32(
        p,
        i32::try_from(new_denom)?,
    ).into_approximation(replen);
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
    let int_adic = RAdic::from_i32(
        p,
        i32::try_from(int_term)?,
    );
    let repeat_adic = RAdic::new(p, vec![], repeating);
    Ok(int_adic + repeat_adic)

}

fn invert_unit_z(z: &ZAdic) -> AdicResult<ZAdic> {

    let p = z.p();
    let AdicValuation::Finite(significance) = z.significance() else {
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
    let neg_divisor = (-z).into_digits().take(significance).collect::<Vec<_>>();

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
