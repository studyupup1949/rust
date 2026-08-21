use num::{traits::Euclid, ToPrimitive, Zero};
use crate::{
    divisible::Composite,
    error::{AdicError, AdicResult},
    normed::{UltraNormed, Valuation, ValuationRing},
};



/// A structure with digits that can be accessed
pub trait HasDigits {

    /// Type for the digits' index,
    ///  e.g. `usize` for [`EAdic`](crate::EAdic) or `isize` for [`QAdic`](crate::QAdic)
    type DigitIndex: ValuationRing + Euclid;

    /// Number of possibilities for digits
    ///
    /// ```
    /// # use adic::{divisible::Divisible, num_adic::{MAdic, PowAdic}, traits::HasDigits, UAdic};
    /// assert_eq!(7, UAdic::new(7, vec![1, 2, 3]).base().value32());
    /// assert_eq!(25, PowAdic::new(UAdic::new(5, vec![3, 2, 1]), 2).base().value32());
    /// assert_eq!(10, MAdic::new([
    ///     PowAdic::new(UAdic::new(2, vec![0, 1]), 1),
    ///     PowAdic::new(UAdic::new(5, vec![3, 2]), 1),
    /// ]).base().value32());
    /// ```
    fn base(&self) -> Composite;

    /// Minimum digit index, possibly zero for positive valuation numbers.
    /// This is the index where the first digit of `[digits](Self::digits)` starts.
    ///
    /// ```
    /// # use adic::{normed::Valuation, traits::HasDigits, QAdic, UAdic};
    /// assert_eq!(Valuation::Finite(0), UAdic::new(5, vec![2]).min_index());
    /// assert_eq!(Valuation::Finite(0), UAdic::new(5, vec![0, 0, 2]).min_index());
    /// assert_eq!(Valuation::Finite(0), QAdic::new(UAdic::new(5, vec![2]), 2).min_index());
    /// assert_eq!(Valuation::Finite(-2), QAdic::new(UAdic::new(5, vec![2]), -2).min_index());
    /// ```
    fn min_index(&self) -> Valuation<Self::DigitIndex>;

    /// The number of digits this number ultimately has, finite or infinite.
    /// Returns `num-valuation` if `valuation` is negative and `num` if it is positive.
    ///
    /// ```
    /// # use adic::{normed::Valuation, traits::{AdicPrimitive, HasDigits}, EAdic, QAdic, UAdic};
    /// assert_eq!(Valuation::Finite(3), UAdic::new(5, vec![2, 1, 3, 0]).num_digits());
    /// assert_eq!(Valuation::Finite(0), UAdic::zero(5).num_digits());
    /// assert_eq!(Valuation::PosInf, EAdic::new_repeating(5, vec![2, 1], vec![3, 0]).num_digits());
    /// assert_eq!(Valuation::Finite(3), QAdic::new(UAdic::new(5, vec![2, 1, 3, 0]), -2).num_digits());
    /// assert_eq!(Valuation::Finite(5), QAdic::new(UAdic::new(5, vec![2, 1, 3, 0]), 2).num_digits());
    /// assert_eq!(Valuation::Finite(0), QAdic::new(UAdic::zero(5), 2).num_digits());
    /// assert_eq!(Valuation::PosInf, QAdic::new(EAdic::new_repeating(5, vec![2, 1], vec![3, 0]), -2).num_digits());
    /// ```
    fn num_digits(&self) -> Valuation<usize>;

    /// Test if this has a finite number of digits
    ///
    /// ```
    /// # use adic::{traits::HasDigits, EAdic, QAdic, UAdic};
    /// assert!(UAdic::new(5, vec![2, 3, 1, 2, 3, 1]).has_finite_digits());
    /// assert!(!EAdic::new_repeating(5, vec![2, 3, 1], vec![2, 1]).has_finite_digits());
    /// assert!(QAdic::new(UAdic::new(5, vec![2, 1, 3, 0]), -2).has_finite_digits());
    /// assert!(!QAdic::new(EAdic::new_repeating(5, vec![2, 1], vec![3, 0]), -2).has_finite_digits());
    /// ```
    fn has_finite_digits(&self) -> bool {
        !matches!(self.num_digits(), Valuation::PosInf)
    }


    /// Gets the digit at this coefficient of p^n; error if it is beyond known digits (certainty)
    ///
    /// ```
    /// # use adic::{error::AdicError, traits::HasDigits, QAdic, UAdic, ZAdic};
    /// let u = UAdic::new(5, vec![2, 1, 3]);
    /// assert_eq!([Ok(2), Ok(3), Ok(0)], [u.digit(0), u.digit(2), u.digit(4)]);
    /// let z = ZAdic::new_approx(5, 4, vec![2, 1, 3]);
    /// assert_eq!([Ok(2), Ok(3)], [z.digit(0), z.digit(2)]);
    /// assert!(matches!(z.digit(4), Err(AdicError::InappropriatePrecision(_))));
    /// let u = QAdic::new(UAdic::new(5, vec![2, 1, 3]), -1);
    /// assert_eq!([Ok(2), Ok(1), Ok(3), Ok(0)], [u.digit(-1), u.digit(0), u.digit(1), u.digit(2)]);
    /// let z = QAdic::new(ZAdic::new_approx(5, 3, vec![2, 1, 3]), -1);
    /// assert_eq!([Ok(2), Ok(1), Ok(3)], [z.digit(-1), z.digit(0), z.digit(1)]);
    /// assert!(matches!(z.digit(2), Err(AdicError::InappropriatePrecision(_))));
    /// ```
    fn digit(&self, n: Self::DigitIndex) -> AdicResult<u32>;

    /// Returns the digit in the zeroth position or Err if it is beyond known digits (certainty)
    ///
    /// ```
    /// # use adic::{error::AdicError, traits::HasDigits, QAdic, UAdic, ZAdic};
    /// assert_eq!(Ok(2), UAdic::new(5, vec![2, 3, 1]).digit0());
    /// assert!(matches!(ZAdic::empty(5).digit0(), Err(AdicError::InappropriatePrecision(_))));
    /// assert_eq!(Ok(3), QAdic::new(UAdic::new(5, vec![2, 3, 1]), -1).digit0());
    /// assert_eq!(Ok(0), QAdic::new(UAdic::new(5, vec![2, 3, 1]), -4).digit0());
    /// assert!(matches!(
    ///     QAdic::new(ZAdic::new_approx(5, 2, vec![1, 2]), -3).digit0(),
    ///     Err(AdicError::InappropriatePrecision(_))
    /// ));
    /// ```
    fn digit0(&self) -> AdicResult<u32> {
        self.digit(Self::DigitIndex::zero())
    }

    /// Digits iterator for this object, starting from [`min_index`](Self::min_index)
    ///
    /// ```
    /// # use adic::{traits::HasDigits, EAdic, QAdic, UAdic};
    /// let u = UAdic::new(5, vec![2, 1, 3]);
    /// assert_eq!(vec![2, 1, 3], u.digits().collect::<Vec<_>>());
    /// let r = EAdic::new_repeating(5, vec![2, 1], vec![3]);
    /// assert_eq!(vec![2, 1, 3, 3, 3, 3], r.digits().take(6).collect::<Vec<_>>());
    /// let q = QAdic::new(UAdic::new(5, vec![2, 1, 3]), -1);
    /// assert_eq!(vec![2, 1, 3], q.digits().collect::<Vec<_>>());
    /// let q = QAdic::new(UAdic::new(5, vec![2, 1, 3]), 1);
    /// assert_eq!(vec![0, 2, 1, 3], q.digits().collect::<Vec<_>>());
    /// ```
    fn digits(&self) -> impl Iterator<Item=u32>;

    /// Flips the digit indices from positive to negative and returns the corresponding `f64`.
    /// E.g. if this is an adic number, it flips the digits around its decimal point and
    ///  returns the value as a real number.
    ///
    /// ```
    /// # use assertables::assert_approx_eq;
    /// # use adic::{traits::HasDigits, EAdic, QAdic};
    /// // 10._5 => 0.01_5 => 0.04
    /// assert_approx_eq!(EAdic::new(5, vec![0, 1]).real_projection().unwrap(), 0.04);
    /// // ...444444.4_5 => 4.44444...._5 => 5
    /// assert_approx_eq!(QAdic::new(EAdic::new_repeating(5, vec![], vec![4]), -1).real_projection().unwrap(), 5.0);
    /// // 0._5 => 0
    /// assert_approx_eq!(EAdic::new(5, vec![]).real_projection().unwrap(), 0.0);
    /// ```
    fn real_projection(&self) -> AdicResult<f64> {

        let zero = Self::DigitIndex::zero();
        let input_offset = match self.min_index() {
            Valuation::Finite(v) if v < zero => (zero - v).try_into_usize()?,
            Valuation::Finite(_) => 0,
            Valuation::PosInf => panic!("Valuation cannot be infinite for the input"),
        };

        // Add 1 so we flip around the decimal point instead of the digit
        let input_offset = -isize::try_from(input_offset)? + 1;
        let inverse_base = 1.0 / f64::from(u32::from(self.base()));
        let offset_power = inverse_base.powf(f64::from(i32::try_from(input_offset)?));


        // epsilon > base.powf(-i)
        // i > -log_base(epsilon)
        let log_eps = f64::EPSILON.log(f64::from(u32::from(self.base())));
        let num_terms = (-log_eps).ceil().to_usize().ok_or(AdicError::BadConversion)?;

        let projected: f64 = self.digits()
            .take(num_terms)
            .enumerate()
            .map(|(i, d)| Ok(f64::from(d)*inverse_base.powf(f64::from(u32::try_from(i)?))))
            .collect::<AdicResult<Vec<_>>>()?
            .into_iter()
            .sum();

        Ok(projected * offset_power)
    }

}



/// A structure that has digits with a concept of certain and uncertain digits
pub trait HasApproximateDigits: HasDigits {

    /// The index of the first unknown digit for this number: `v(...0021.30_5) = 4`
    ///
    /// Returns a [`Valuation`].
    /// Returns `PosInf` for an exact numbers and `Finite(v)` for an approximate number with digits to the `v-th` valuation.
    ///
    /// ```
    /// # use adic::{normed::Valuation, traits::HasApproximateDigits, QAdic, UAdic, ZAdic};
    /// assert_eq!(Valuation::Finite(6), ZAdic::new_approx(5, 6, vec![0, 3, 1, 2]).certainty());
    /// assert_eq!(Valuation::PosInf, UAdic::new(5, vec![0, 3, 1, 2]).certainty());
    /// assert_eq!(Valuation::Finite(4), QAdic::new(ZAdic::new_approx(5, 6, vec![0, 3, 1, 2]), -2).certainty());
    /// assert_eq!(Valuation::Finite(-2), QAdic::new(ZAdic::new_approx(5, 2, vec![0, 3]), -4).certainty());
    /// assert_eq!(Valuation::PosInf, QAdic::new(UAdic::new(5, vec![0, 3, 1, 2]), -2).certainty());
    /// ```
    fn certainty(&self) -> Valuation<Self::DigitIndex>;

    /// The number is completely uncertain, has no known digits
    ///
    /// ```
    /// # use adic::{traits::{AdicPrimitive, HasApproximateDigits}, QAdic, ZAdic};
    /// assert!(!ZAdic::new_approx(5, 6, vec![0, 3, 1, 2]).has_no_certainty());
    /// assert!(!ZAdic::new_approx(5, 1, vec![3]).has_no_certainty());
    /// assert!(ZAdic::empty(5).has_no_certainty());
    /// assert!(!QAdic::new(ZAdic::new_approx(5, 6, vec![0, 3, 1, 2]), -2).has_no_certainty());
    /// assert!(!QAdic::new(ZAdic::new_approx(5, 1, vec![3]), -2).has_no_certainty());
    /// assert!(QAdic::new(ZAdic::empty(5), -2).has_no_certainty());
    /// assert!(!QAdic::new(ZAdic::zero(5), 0).has_no_certainty());
    /// ```
    fn has_no_certainty(&self) -> bool {
        if let (Valuation::Finite(v), Valuation::Finite(c)) = (self.min_index(), self.certainty()) {
            v >= c
        } else {
            false
        }
    }

    /// The number is completely certain, has no unknown digits
    ///
    /// ```
    /// # use adic::{traits::HasApproximateDigits, UAdic, QAdic, ZAdic};
    /// assert!(UAdic::new(5, vec![2, 3, 1, 2, 3, 1]).is_certain());
    /// assert!(!ZAdic::new_approx(5, 6, vec![2, 3, 1, 2, 3, 1]).is_certain());
    /// assert!(QAdic::new(UAdic::new(5, vec![2, 3, 1, 2, 3, 1]), -2).is_certain());
    /// assert!(!QAdic::new(ZAdic::new_approx(5, 6, vec![2, 3, 1, 2, 3, 1]), -2).is_certain());
    /// ```
    fn is_certain(&self) -> bool {
        matches!(self.certainty(), Valuation::PosInf)
    }

    /// The digital distance between minimum index and maximum (certainty).
    ///
    /// Returns a [`Valuation`].
    /// Returns `Finite(0)` for zero (i.e. for `infinity-infinity`).
    ///
    /// ```
    /// # use adic::{normed::Valuation, traits::HasApproximateDigits, QAdic, UAdic, ZAdic};
    /// assert_eq!(Valuation::PosInf, UAdic::new(5, vec![1]).significance());
    /// assert_eq!(Valuation::PosInf, UAdic::new(5, vec![0, 0, 1]).significance());
    /// assert_eq!(Valuation::Finite(0), UAdic::new(5, vec![0]).significance());
    /// assert_eq!(Valuation::Finite(4), ZAdic::new_approx(5, 4, vec![1, 0, 0, 0]).significance());
    /// assert_eq!(Valuation::Finite(2), ZAdic::new_approx(5, 4, vec![0, 0, 1, 0]).significance());
    /// assert_eq!(Valuation::Finite(0), ZAdic::new_approx(5, 4, vec![0, 0, 0, 0]).significance());
    /// assert_eq!(Valuation::PosInf, QAdic::new(UAdic::new(5, vec![0, 0, 1]), -1).significance());
    /// assert_eq!(Valuation::Finite(4), QAdic::new(ZAdic::new_approx(5, 4, vec![1, 0, 0, 0]), -8).significance());
    /// assert_eq!(Valuation::Finite(2), QAdic::new(ZAdic::new_approx(5, 4, vec![0, 0, 1, 0]), 4).significance());
    /// assert_eq!(Valuation::Finite(0), QAdic::new(ZAdic::new_approx(5, 4, vec![0, 0, 0, 0]), 4).significance());
    /// ```
    fn significance(&self) -> Valuation<Self::ValuationRing>
    where
    Self: UltraNormed<ValuationRing = Self::DigitIndex>,
    Self::ValuationRing: std::ops::Sub<Output=Self::ValuationRing> {
        match (self.certainty(), self.valuation()) {
            (_, Valuation::PosInf) => Valuation::zero(),
            (Valuation::PosInf, Valuation::Finite(_)) => Valuation::PosInf,
            (Valuation::Finite(c), Valuation::Finite(v)) => {
                let s = (c - v);
                Valuation::Finite(s)
            },
        }
    }

}
