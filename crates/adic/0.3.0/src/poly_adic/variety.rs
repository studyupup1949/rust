//! Adic integer algebraic variety

use std::{cmp::Ordering, fmt::Display};
use itertools::{EitherOrBoth, Itertools};
use crate::{AdicInteger, ZAdic};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An algebraic variety, a set of integer "roots" to an algebraic equation ([`zadic_variety`](crate::zadic_variety))
///
/// ```
/// # use adic::{ZAdic, ZAdicVariety};
/// let z = ZAdicVariety::new(5, vec![
///     ZAdic::new_approx(5, 6, vec![0, 1, 2, 3]),
///     ZAdic::new_exact_pos(5, vec![4, 3])
/// ]);
/// assert_eq!("variety(...003210._5, 34._5)", z.to_string());
/// ```
///
/// Often known as the "solutions" of the equation.
///
/// E.g. `x^2 - 4 = 0` has an associated variety of `{2, -2}` (in both the reals and all p-adics).
///
/// E.g. `x^2 - 2 = 0` has a non-integer variety `{1.414..., -1.414...}` in the reals,
/// integer variety `{...6213._5, ...0454._5}` in the 7-adics,
/// and no solutions/variety in the 5-adics (indecomposable and irreducible).
pub struct ZAdicVariety {
    p: u32,
    elements: Vec<ZAdic>,
}


impl ZAdicVariety {

    /// Prime for this adic variety
    pub fn p(&self) -> u32 {
        self.p
    }

    /// Number of roots in variety
    pub fn num_roots(&self) -> usize {
        self.elements.len()
    }

    /// Iterator reference for the roots of this variety
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_variety};
    /// let v = zadic_variety!(5, 3, [[1, 2, 3], [3, 2, 1]]);
    /// assert_eq!(
    ///     vec![&zadic_approx!(5, 3, [1, 2, 3]), &zadic_approx!(5, 3, [3, 2, 1])],
    ///     v.roots().collect::<Vec<_>>()
    /// );
    /// ```
    pub fn roots(&self) -> impl Iterator<Item=&ZAdic> {
        self.elements.iter()
    }

    /// Iterator for the roots of this variety
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_variety};
    /// let v = zadic_variety!(5, 3, [[1, 2, 3], [3, 2, 1]]);
    /// assert_eq!(
    ///     vec![zadic_approx!(5, 3, [1, 2, 3]), zadic_approx!(5, 3, [3, 2, 1])],
    ///     v.into_roots().collect::<Vec<_>>()
    /// );
    /// ```
    pub fn into_roots(self) -> impl Iterator<Item=ZAdic> {
        self.elements.into_iter()
    }

    /// Do no roots exist
    ///
    /// ```
    /// # use adic::zadic_variety;
    /// let v = zadic_variety!(5, 3, [[1, 2, 3], [3, 2, 1]]);
    /// assert!(!v.is_empty());
    /// let v = zadic_variety!(5, 3, []);
    /// assert!(v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Create an adic variety with the given roots/solutions
    pub fn new(p: u32, roots: Vec<ZAdic>) -> Self {

        let sorted_roots = roots.into_iter().sorted_by(|z1, z2| {
            if z1 == z2 {
                Ordering::Equal
            } else {
                match z1.digits().zip_longest(z2.digits()).find(
                    |dd| dd.as_ref().both().is_none_or(|(l, r)| l != r)
                ) {
                    None => {
                        if z1.is_certain() && !z2.is_certain() {
                            Ordering::Less
                        } else if !z1.is_certain() && z2.is_certain() {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    },
                    Some(EitherOrBoth::Left(_)) => Ordering::Greater,
                    Some(EitherOrBoth::Right(_)) => Ordering::Less,
                    Some(EitherOrBoth::Both(d1, d2)) => d1.cmp(d2),
                }
            }
        }).collect::<Vec<_>>();

        Self {
            p,
            elements: sorted_roots,
        }

    }

    /// Create an empty adic variety
    pub fn empty(p: u32) -> Self {
        Self {
            p,
            elements: vec![],
        }
    }

}


impl Display for ZAdicVariety {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "variety(empty({}-adic))", self.p)
        } else {
            write!(f, "variety({})", self.elements.iter().map(ToString::to_string).join(", "))
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::{zadic_approx, zadic_exact_pos, zadic_variety, ZAdicVariety};

    #[test]
    fn roots_sorted() {

        let var = zadic_variety!(5, 2, [[3, 2], [3, 1], [4, 4], [0, 2], [4, 0]]);
        assert_eq!(zadic_variety!(5, 2, [[0, 2], [3, 1], [3, 2], [4, 0], [4, 4]]), var);
        assert_eq!(
            vec![
                zadic_approx!(5, 2, [0, 2]), zadic_approx!(5, 2, [3, 1]), zadic_approx!(5, 2, [3, 2]),
                zadic_approx!(5, 2, [4, 0]), zadic_approx!(5, 2, [4, 4]),
            ],
            var.into_roots().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![zadic_approx!(5, 2, [2, 1]), zadic_approx!(5, 3, [2, 1, 0])],
            ZAdicVariety::new(
                5, vec![zadic_approx!(5, 3, [2, 1, 0]), zadic_approx!(5, 2, [2, 1])]
            ).into_roots().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![zadic_exact_pos!(5, [2, 1]), zadic_approx!(5, 2, [2, 1])],
            ZAdicVariety::new(
                5, vec![zadic_approx!(5, 2, [2, 1]), zadic_exact_pos!(5, [2, 1])]
            ).into_roots().collect::<Vec<_>>()
        );

    }

    #[test]
    fn display() {

        let var = ZAdicVariety::empty(5);
        assert_eq!("variety(empty(5-adic))", var.to_string());

        let var = zadic_variety!(5, 6, [[0, 0, 0, 0, 0, 0], [1, 2, 3, 0, 1, 2]]);
        assert_eq!("variety(...000000._5, ...210321._5)", var.to_string());

        let var = ZAdicVariety::new(5, vec![zadic_exact_pos!(5, [1, 0, 3, 0, 0, 0]), zadic_approx!(5, 6, [1, 2, 3, 0, 1, 2])]);
        assert_eq!("variety(301._5, ...210321._5)", var.to_string());

    }

}
