//! Adic integer algebraic variety

use std::fmt::Display;
use itertools::Itertools;
use crate::ZAdic;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An algebraic variety, a set of integer "roots" to an algebraic equation ([`zadic_variety`](crate::zadic_variety))
///
/// ```
/// # use adic::{ZAdic, ZAdicVariety};
/// let z = ZAdicVariety::new(5, vec![
///     ZAdic::new_approx(5, 6, vec![0, 1, 2, 3]),
///     ZAdic::new_exact(5, vec![4, 3])
/// ]);
/// assert_eq!("variety(...003210._5, 34._5)", z.to_string());
/// ```
///
/// Often, these are called the "roots" or "solutions" of the equation.
///
/// E.g. `x^2 - 4 = 0` has an associated variety of `{2, -2}` (in both the reals and all p-adics).
///
/// E.g. `x^2 - 2 = 0` has a non-integer variety `{1.414..., -1.414...}` in the reals,
/// integer variety `{...6213._5, ...0454._5}` in the 7-adics,
/// and no solutions/variety in the 5-adics (indecomposable).
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
    pub fn num_roots(&self) -> u32 {
        self.elements.len() as u32
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
        Self {
            p,
            elements: roots,
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
    use crate::{zadic_approx, zadic_exact, zadic_variety, ZAdicVariety};

    #[test]
    fn test_display() {

        let var = ZAdicVariety::empty(5);
        assert_eq!("variety(empty(5-adic))", var.to_string());

        let var = zadic_variety!(5, 6, [[0, 0, 0, 0, 0, 0], [1, 2, 3, 0, 1, 2]]);
        assert_eq!("variety(...000000._5, ...210321._5)", var.to_string());

        let var = ZAdicVariety::new(5, vec![zadic_exact!(5, [1, 0, 3, 0, 0, 0]), zadic_approx!(5, 6, [1, 2, 3, 0, 1, 2])]);
        assert_eq!("variety(301._5, ...210321._5)", var.to_string());

    }

}
