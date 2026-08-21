use crate::{AdicFraction, AdicInteger, AdicNumber, AdicValuation, IAdic, Prime, UAdic, ZAdic};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Adic number, including fractional digits ([`qadic`](crate::qadic))
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
/// # use adic::{qadic, radic, uadic, zadic_approx, AdicInteger, AdicValuation, QAdic};
/// let twenty_three_and_11_25 = QAdic::new(uadic!(5, [1, 2, 3, 4]), AdicValuation::Finite(-2));
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
    pub (super) valuation: AdicValuation<isize>,
}


impl<A> QAdic<A>
where A: AdicInteger {

    /// Create an adic number with the given digits and certainty
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new<V>(adic_int: A, valuation: V) -> Self
    where V: Into<AdicValuation<isize>> {

        let p = adic_int.p();
        let (adic_unit, int_valuation) = adic_int.into_unit_and_valuation();
        let adjusted_valuation = valuation.into() + int_valuation.convert().expect("valuation conversion");
        match (adic_unit, adjusted_valuation) {
            (Some(unit), AdicValuation::Finite(v)) => Self {
                adic_unit: unit,
                valuation: v.into(),
            },
            _ => Self::zero(p),
        }

    }

}


impl QAdic<ZAdic> {

    /// Create the empty `QAdic<ZAdic>` with valuation `v`
    pub fn empty<P, V>(p: P, v: V) -> Self
    where P: Into<Prime>, V: Into<AdicValuation<isize>> {
        QAdic::new(ZAdic::empty(p), v)
    }

}


impl<A> AdicNumber for QAdic<A>
where A: AdicInteger {

    fn zero<P>(p: P) -> Self
    where P: Into<Prime> {
        Self {
            adic_unit: A::zero(p),
            valuation: AdicValuation::PosInf,
        }
    }
    fn one<P>(p: P) -> Self
    where P: Into<Prime> {
        Self {
            adic_unit: A::one(p),
            valuation: AdicValuation::Finite(0),
        }
    }
    fn p(&self) -> Prime {
        self.adic_unit.p()
    }

}


impl<A> AdicFraction for QAdic<A>
where A: AdicInteger {

    type AI = A;

    fn unit_ref(&self) -> &A {
        &self.adic_unit
    }

}


impl<A> From<UAdic> for QAdic<A>
where A: AdicInteger {
    fn from(value: UAdic) -> Self {
        QAdic::new(A::from(value), 0)
    }
}

impl<A> From<(UAdic, AdicValuation<isize>)> for QAdic<A>
where A: AdicInteger {
    fn from((a, val): (UAdic, AdicValuation<isize>)) -> Self {
        QAdic::new(A::from(a), val)
    }
}

impl<A> From<IAdic> for QAdic<A>
where A: AdicInteger + From<IAdic> {
    fn from(value: IAdic) -> Self {
        QAdic::new(A::from(value), 0)
    }
}


#[cfg(test)]
mod tests {
    use crate::{
        qadic, uadic, zadic_approx,
        AdicApproximate, AdicSized, AdicValuation, HasDigits, QAdic, ZAdic,
    };

    use crate::num_adic::test_util::qu::*;


    #[test]
    fn adjusts_validation() {
        assert_eq!(qadic!(uadic!(5, [2]), 5), qadic!(uadic!(5, [0, 0, 2]), 3));
        assert_eq!(qadic!(uadic!(5, [2]), -3), qadic!(uadic!(5, [0, 0, 2]), -5));
        assert_eq!(one(), five_fifth());
    }

    #[test]
    fn adjusts_certainty() {
        assert_eq!(qadic!(zadic_approx!(5, 2, [2, 0]), 5), qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), 3));
        assert_eq!(qadic!(zadic_approx!(5, 2, [2, 0]), -3), qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), -5));
        assert_eq!(AdicValuation::Finite(7), qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), 3).certainty());
        assert_eq!(AdicValuation::Finite(2), qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), 3).significance());
    }

    #[test]
    fn min_index() {
        assert_eq!(AdicValuation::Finite(0), qadic!(uadic!(5, [2]), 0).min_index());
        assert_eq!(AdicValuation::Finite(0), qadic!(uadic!(5, [2]), 2).min_index());
        assert_eq!(AdicValuation::Finite(-2), qadic!(uadic!(5, [2]), -2).min_index());
    }

    #[test]
    fn empty_qz_adic() {

        let zempty = || ZAdic::empty(5);
        assert_eq!(
            (Some(zempty()), AdicValuation::Finite(0)),
            QAdic::empty(5, 0).into_unit_and_valuation()
        );
        assert_eq!(
            (Some(zempty()), AdicValuation::Finite(1)),
            QAdic::empty(5, 1).into_unit_and_valuation()
        );
        assert_eq!(
            (Some(zempty()), AdicValuation::Finite(-1)),
            QAdic::new(ZAdic::empty(5), -1).into_unit_and_valuation()
        );
        assert_eq!(
            (Some(zempty()), AdicValuation::Finite(1)),
            qadic!(zadic_approx!(5, 2, [0, 0]), -1).into_unit_and_valuation()
        );

    }

}
