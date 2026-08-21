use crate::{adic_valid, AdicInteger, QAdicValuation, UAdic, ZAdic};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Adic number ([`qadic`](crate::qadic))
///
/// The struct holds an adic integer (specifically, an adic unit), and a valuation.
/// Digitally, there are `-valuation` digits to the right of the decimal.
/// With this, you can represent any adic number.
///
/// The adic integer is generic and so can be e.g.
/// - natural number [`UAdic`](crate::UAdic)
/// - signed integer [`IAdic`](crate::IAdic)
/// - unit fraction [`RAdic`](crate::RAdic)
/// - approximate number [`ZAdic`](crate::ZAdic)
///
/// ```
/// # use adic::{qadic, radic, uadic, zadic_approx, AdicInteger, QAdic, QAdicValuation};
/// let twenty_three_and_11_25 = QAdic::new(uadic!(5, [1, 2, 3, 4]), QAdicValuation::Finite(-2));
/// assert_eq!("43.21_5", twenty_three_and_11_25.to_string());
/// let fifty = qadic!(uadic!(5, [0, 2]), 1);
/// assert_eq!("200._5", fifty.to_string());
/// let neg_one_tenth = qadic!(radic!(5, [], [2]), -1);
/// assert_eq!("(2).2_5", neg_one_tenth.to_string());
/// let sqrt_neg_one_fifth = qadic!(zadic_approx!(5, 6, [2, 1, 2, 1, 3, 4]), -1);
/// assert_eq!("...43121.2_5", sqrt_neg_one_fifth.to_string());
/// assert_eq!(
///     qadic!(uadic!(5, [1, 2, 4, 1, 1]), -2),
///     qadic!(uadic!(5, [1, 2, 3, 4]), -2) + qadic!(uadic!(5, [1, 2]), 0)
/// );
/// assert_eq!(
///     qadic!(uadic!(5, [2, 2]), -3),
///     qadic!(uadic!(5, [3]), -2) * qadic!(uadic!(5, [4]), -1)
/// );
/// ```
pub struct QAdic<A>
where A: AdicInteger {
    pub (super) adic_unit: A,
    pub (super) valuation: QAdicValuation,
}


impl<A> QAdic<A>
where A: AdicInteger {

    /// Create an adic number with the given digits and certainty
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new(adic_int: A, valuation: QAdicValuation) -> Self {

        let p = adic_int.p();
        let (adic_unit, int_valuation) = adic_int.into_unit_and_valuation();
        match valuation + int_valuation.into() {
            QAdicValuation::PosInf => Self::zero(p),
            QAdicValuation::Finite(actual_valuation) => {
                Self {
                    adic_unit,
                    valuation: QAdicValuation::Finite(actual_valuation),
                }
            }
        }

    }

    /// Create the zero adic number
    pub fn zero(p: u32) -> Self {
        adic_valid::validate_p(p);
        Self {
            adic_unit: A::zero(p),
            valuation: QAdicValuation::PosInf,
        }
    }

    /// Create the one adic number
    pub fn one(p: u32) -> Self {
        adic_valid::validate_p(p);
        Self {
            adic_unit: A::one(p),
            valuation: QAdicValuation::Finite(0),
        }
    }

    /// Create a `QAdic` representing a power of p
    ///
    /// ```
    /// # use adic::{qadic, uadic, QAdic, QAdicValuation, UAdic};
    /// assert_eq!(qadic!(uadic!(5, [0, 0, 0, 1]), 0), QAdic::p_power(5, QAdicValuation::Finite(3)));
    /// assert_eq!(qadic!(uadic!(5, [1]), -3), QAdic::p_power(5, QAdicValuation::Finite(-3)));
    /// assert_eq!(QAdic::<UAdic>::zero(5), QAdic::p_power(5, QAdicValuation::PosInf));
    /// ```
    pub fn p_power(p: u32, power: QAdicValuation) -> Self {
        Self::new(A::one(p), power)
    }

    /// Prime for this adic
    pub fn p(&self) -> u32 {
        self.adic_unit.p()
    }

    /// The adic unit for this number: `u(a/b p^v) = a/b`
    ///
    /// In the digital representation, the adic integer resulting from moving the first nonzero digit
    ///  directly to the left of the decimal point.
    ///
    /// Returns an [`AdicInteger`]. Returns `A::zero` if `QAdic` is zero.
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicInteger, QAdic, QAdicValuation, UAdic};
    /// let q = qadic!(uadic!(5, [1, 2]), 4);
    /// assert_eq!(uadic!(5, [1, 2]), *q.unit());
    /// let q = qadic!(uadic!(5, [1, 2]), -4);
    /// assert_eq!(uadic!(5, [1, 2]), *q.unit());
    /// let q = qadic!(uadic!(5, [0, 0, 1]), -4);
    /// assert_eq!(uadic!(5, [1]), *q.unit());
    /// let q = QAdic::<UAdic>::zero(5);
    /// assert_eq!(UAdic::zero(5), *q.unit());
    /// ```
    pub fn unit(&self) -> &A {
        &self.adic_unit
    }

    /// The adic valuation for this number: `v(a/b p^v) = v`
    ///
    /// In the digital representation, the number of zeroes to the left (positive)
    ///  or the number of digits to the right (negative) of the decimal point.
    ///
    /// Returns a [`QAdicValuation`].
    /// Returns `PosInf` for zero and `Finite(v)` otherwise.
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicInteger, QAdic, QAdicValuation, UAdic};
    /// let q = qadic!(uadic!(5, [1, 2]), 4);
    /// assert_eq!(QAdicValuation::Finite(4), q.valuation());
    /// let q = qadic!(uadic!(5, [1, 2]), -4);
    /// assert_eq!(QAdicValuation::Finite(-4), q.valuation());
    /// let q = qadic!(uadic!(5, [0, 0, 1]), -4);
    /// assert_eq!(QAdicValuation::Finite(-2), q.valuation());
    /// let q = QAdic::<UAdic>::zero(5);
    /// assert_eq!(QAdicValuation::PosInf, q.valuation());
    /// ```
    pub fn valuation(&self) -> QAdicValuation {
        self.valuation
    }

    /// Transform into the adic unit and valuation form; transforms zero into `(0, PosInf)`
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicInteger, QAdic, QAdicValuation, UAdic};
    /// let (unit, valuation) = qadic!(uadic!(5, [0, 3, 1]), 4).unit_and_valuation();
    /// assert_eq!((uadic!(5, [3, 1]), QAdicValuation::Finite(5)), (unit, valuation));
    /// assert_eq!((UAdic::zero(5), QAdicValuation::PosInf), QAdic::<UAdic>::zero(5).unit_and_valuation());
    /// ```
    pub fn unit_and_valuation(self) -> (A, QAdicValuation) {
        match self.valuation() {
            QAdicValuation::PosInf => (A::zero(self.p()), QAdicValuation::PosInf),
            QAdicValuation::Finite(_) => {
                (self.adic_unit, self.valuation)
            }
        }
    }

    /// Test if it is the zero adic number
    ///
    /// ```
    /// # use adic::{qadic, uadic};
    /// assert!(qadic!(uadic!(5, []), 7).is_zero());
    /// assert!(!qadic!(uadic!(5, [2, 3, 1, 2, 3, 1]), -3).is_zero());
    /// ```
    pub fn is_zero(&self) -> bool {
        self.adic_unit.is_zero()
    }

    /// Approximate an adic number's expansion to n digits.
    /// See also: [`into_approximation`](Self::into_approximation)
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, zadic_approx, AdicInteger};
    /// let q = qadic!(uadic!(5, [1, 3, 2, 1, 2, 1, 2]), -5);
    /// let z = qadic!(zadic_approx!(5, 9, [1, 3, 2, 1, 2, 1, 2, 0, 0]), -5);
    /// let zs = qadic!(zadic_approx!(5, 5, [1, 3, 2, 1, 2]), -5);
    /// assert_eq!(z, q.approximation(9));
    /// assert_eq!(zs, q.approximation(5));
    /// let qr = qadic!(radic!(5, [1, 3], [2, 1]), -5);
    /// assert_eq!(zs, qr.approximation(5));
    /// ```
    pub fn approximation(&self, n: usize) -> QAdic<ZAdic> {
        QAdic::new(self.adic_unit.approximation(n), self.valuation)
    }

    /// Consume `AdicInteger` and get the approximation.
    /// See also: [`approximation`](Self::approximation)
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, zadic_approx, AdicInteger};
    /// let q = qadic!(uadic!(5, [1, 3, 2, 1, 2, 1, 2]), -5);
    /// let z = qadic!(zadic_approx!(5, 9, [1, 3, 2, 1, 2, 1, 2, 0, 0]), -5);
    /// let zs = qadic!(zadic_approx!(5, 5, [1, 3, 2, 1, 2]), -5);
    /// assert_eq!(z, q.clone().into_approximation(9));
    /// assert_eq!(zs, q.clone().into_approximation(5));
    /// let qr = qadic!(radic!(5, [1, 3], [2, 1]), -5);
    /// assert_eq!(zs, qr.into_approximation(5));
    /// ```
    pub fn into_approximation(self, n: usize) -> QAdic<ZAdic> {
        QAdic::new(self.adic_unit.into_approximation(n), self.valuation)
    }

    /// Split `QAdic` at p^n into integer above (as `A`) and remainder (as `UAdic`)
    ///
    /// ```
    /// # use adic::{qadic, radic};
    /// let r = radic!(7, [1, 2], [3, 4, 5]);
    /// assert_eq!("(543)21._7", r.to_string());
    /// let q = qadic!(r, -6);
    /// assert_eq!("(354).354321_7", q.to_string());
    /// let (q_int, q_rem) = q.split(1);
    /// assert_eq!("(435)._7", q_int.to_string());
    /// assert_eq!("4354321._7", q_rem.to_string());
    /// let (q_int, q_rem) = q.split(-5);
    /// assert_eq!("(543)2._7", q_int.to_string());
    /// assert_eq!("1._7", q_rem.to_string());
    /// let (q_int, q_rem) = q.split(-7);
    /// assert_eq!("(543)210._7", q_int.to_string());
    /// assert_eq!("0._7", q_rem.to_string());
    /// ```
    pub fn split(&self, n: isize) -> (A, UAdic) {
        // Note: maybe should actually return the fraction as a QAdic
        let p = self.adic_unit.p();
        match self.valuation {
            QAdicValuation::PosInf => (A::zero(p), UAdic::zero(p)),
            QAdicValuation::Finite(valuation) => {
                let adj_val = valuation - n;
                let pos_val = adj_val.unsigned_abs();
                if adj_val < 0 {
                    let (before_decimal, after_decimal) = self.adic_unit.split(pos_val);
                    (after_decimal, before_decimal)
                } else {
                    let adic_int = self.adic_unit.clone() * A::p_power(p, pos_val);
                    (adic_int, UAdic::zero(p))
                }
            }
        }
    }

    /// Split `QAdic` into integer and remainder (as a `UAdic` not a `QAdic` fraction)
    ///
    /// ```
    /// # use adic::{qadic, radic};
    /// let r = radic!(7, [1, 2], [3, 4, 5]);
    /// assert_eq!("(543)21._7", r.to_string());
    /// let q = qadic!(r, -6);
    /// assert_eq!("(354).354321_7", q.to_string());
    /// let (q_int, q_rem) = q.int_and_rem();
    /// assert_eq!("(354)._7", q_int.to_string());
    /// assert_eq!("354321._7", q_rem.to_string());
    /// ```
    pub fn int_and_rem(&self) -> (A, UAdic) {
        // Note: maybe should actually return the fraction as a QAdic
        self.split(0)
    }

}



#[cfg(test)]
mod tests {
    use crate::{qadic, uadic};

    use crate::num_adic::test_util::qu::*;


    #[test]
    fn adjusts_validation() {
        assert_eq!(qadic!(uadic!(5, [2]), 5), qadic!(uadic!(5, [0, 0, 2]), 3));
        assert_eq!(qadic!(uadic!(5, [2]), -3), qadic!(uadic!(5, [0, 0, 2]), -5));
        assert_eq!(one(), five_fifth());
    }

}
