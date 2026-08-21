use std::cmp::Ordering;
use num::traits::Inv;
use crate::{adic_valid, AdicError};
use super::{AdicInteger, QAdic, QAdicValuation, RAdic, ZAdic, ZAdicValuation};


#[derive(Debug, Clone)]
/// Lazy struct holding onto adic number division for further information,
///  e.g. either exact or approximate division and how much precision.
/// Note this will TRUNCATE, as it is integer division.
/// Use [`QAdic`](crate::QAdic) and [`LazyQDiv`](crate::LazyQDiv) for full number division.
pub struct LazyIntDiv<A>
where A: AdicInteger {
    a: A,
    b: A,
}


impl<A> LazyIntDiv<A>
where A: AdicInteger {

    /// Create a new `LazyIntDiv`. Probably should only be used from `AdicInteger` div methods.
    pub fn new(a: A, b: A) -> Self {
        adic_valid::validate_p(a.p());
        adic_valid::validate_mono_character(a.p(), b.p());
        Self {
            a,
            b,
        }
    }

    /// Perform the approximate division with given precision, giving a `ZAdic`.
    /// Note the input should have a `significance` of at least `precision` to succeed.
    /// E.g. dividing by `zadic_approx!(5, 4, [0, 0, 0, 0])` will always return an error
    ///  (unless `a` is zero) because it is LIKE dividing by zero.
    ///
    /// # Errors
    /// Returns an error if the numbers are not precise enough or the denominator is zero
    ///
    /// ```
    /// # use adic::{iadic_neg, iadic_pos, zadic_approx, AdicError, AdicInteger, IAdic};
    /// let neg_one = iadic_neg!(5, []);
    /// let four = iadic_pos!(5, [4]);
    /// let intermediate_div = &neg_one / four;
    /// assert_eq!(Ok(zadic_approx!(5, 6, [1, 1, 1, 1, 1, 1])), intermediate_div.approx(6));
    /// let bad_div = neg_one / IAdic::zero(5);
    /// assert_eq!(Err(AdicError::DivideByZero), bad_div.approx(6));
    /// ```
    pub fn approx(self, precision: usize) -> Result<ZAdic, AdicError> {

        let p = self.a.p();
        let (ua, va) = self.a.unit_and_valuation();
        let (ub, vb) = self.b.unit_and_valuation();

        match (va, vb) {
            (ZAdicValuation::Finite(val_a), ZAdicValuation::Finite(val_b)) => {

                let in_precision = if val_a >= val_b { precision } else { precision + val_b - val_a };
                let in_prec_v = ZAdicValuation::Finite(in_precision);
                if ua.certainty() < in_prec_v || ub.certainty() < in_prec_v {
                    Err(AdicError::InappropriatePrecision(format!("a and b not precise enough to give {precision} digits")))
                } else {
                    let za = ua.into_approximation(in_precision);
                    let zb = ub.into_approximation(in_precision);
                    let div = za * zb.inv();
                    match val_a.cmp(&val_b) {
                        Ordering::Equal => Ok(div),
                        Ordering::Greater => Ok(div * ZAdic::p_power(p, val_a - val_b)),
                        Ordering::Less => Ok(div.quotient(val_b - val_a)),
                    }
                }

            },
            (ZAdicValuation::PosInf, ZAdicValuation::Finite(_)) => Ok(ZAdic::zero(p)),
            (_, ZAdicValuation::PosInf) => {
                Err(AdicError::DivideByZero)
            },
        }

    }

}


impl<A> LazyIntDiv<A>
where A: AdicInteger, RAdic: From<A> {

    /// Perform the exact division, giving a `RAdic`.
    ///
    /// # Errors
    /// Returns an error if the numbers are inexact or the denominator is zero
    ///
    /// <div class="warning">
    ///
    /// Do not use this method; exact Adic number division is not yet implemented.
    /// Even when implemented, this will not perform nearly as well as [`approx`](Self::approx),
    ///  by the nature of digital rational division.
    ///
    /// </div>
    pub fn exact(self) -> Result<RAdic, AdicError> {

        // NOTE: This is not currently implemented; do not use!

        if self.b.is_zero() {
            Err(AdicError::DivideByZero)
        } else {
            let ra = RAdic::from(self.a);
            let rb = RAdic::from(self.b);
            Ok(ra * rb.inv())
        }

    }

}



#[derive(Debug, Clone)]
/// Lazy struct holding onto adic number division for further information,
///  e.g. either exact or approximate division and how much precision.
pub struct LazyQDiv<A>
where A: AdicInteger {
    a: QAdic<A>,
    b: QAdic<A>,
}


impl<A> LazyQDiv<A>
where A: AdicInteger {

    /// Create a new `LazyIntDiv`. Probably should only be used from `AdicInteger` div methods.
    pub fn new(a: QAdic<A>, b: QAdic<A>) -> Self {
        adic_valid::validate_p(a.p());
        adic_valid::validate_mono_character(a.p(), b.p());
        Self {
            a,
            b,
        }
    }

    /// Perform the approximate division with given precision, giving a `ZAdic`.
    ///
    /// # Errors
    /// Returns an error if the numbers are not precise enough or the denominator is zero
    ///
    /// ```
    /// # use adic::{iadic_neg, iadic_pos, qadic, zadic_approx, AdicError, AdicInteger, IAdic};
    /// let neg_one = qadic!(iadic_neg!(5, []), 0);
    /// let twenty = qadic!(iadic_pos!(5, [4]), 1);
    /// let intermediate_div = &neg_one / twenty;
    /// assert_eq!(Ok(qadic!(zadic_approx!(5, 6, [1, 1, 1, 1, 1, 1]), -1)), intermediate_div.approx(6));
    /// let bad_div = neg_one / qadic!(IAdic::zero(5), 0);
    /// assert_eq!(Err(AdicError::DivideByZero), bad_div.approx(6));
    /// ```
    pub fn approx(self, precision: usize) -> Result<QAdic<ZAdic>, AdicError> {

        let p = self.a.p();
        let (ua, va) = self.a.unit_and_valuation();
        let (ub, vb) = self.b.unit_and_valuation();

        match (va, vb) {
            (QAdicValuation::Finite(val_a), QAdicValuation::Finite(val_b)) => {
                let unit_div = LazyIntDiv::new(ua, ub).approx(precision)?;
                Ok(QAdic::new(unit_div, QAdicValuation::Finite(val_a - val_b)))
            },
            (QAdicValuation::PosInf, QAdicValuation::Finite(_)) => Ok(QAdic::zero(p)),
            (_, QAdicValuation::PosInf) => {
                Err(AdicError::DivideByZero)
            },
        }

    }

}

impl<A> LazyQDiv<A>
where A: AdicInteger, RAdic: From<A> {

    /// Perform the exact division, giving a `RAdic`.
    ///
    /// # Errors
    /// Returns an error if the numbers are inexact or the denominator is zero
    ///
    /// <div class="warning">
    ///
    /// Do not use this method; exact Adic number division is not yet implemented.
    /// Even when implemented, this will not perform nearly as well as [`approx`](Self::approx),
    ///  by the nature of digital rational division.
    ///
    /// </div>
    pub fn exact(self) -> Result<QAdic<RAdic>, AdicError> {

        let p = self.a.p();
        let (ua, va) = self.a.unit_and_valuation();
        let (ub, vb) = self.b.unit_and_valuation();

        match (va, vb) {
            (QAdicValuation::Finite(val_a), QAdicValuation::Finite(val_b)) => {
                let unit_div = LazyIntDiv::new(ua, ub).exact()?;
                Ok(QAdic::new(unit_div, QAdicValuation::Finite(val_a - val_b)))
            },
            (QAdicValuation::PosInf, QAdicValuation::Finite(_)) => Ok(QAdic::zero(p)),
            (_, QAdicValuation::PosInf) => {
                Err(AdicError::DivideByZero)
            },
        }

    }

}
