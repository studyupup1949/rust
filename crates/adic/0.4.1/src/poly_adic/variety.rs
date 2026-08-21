//! Adic integer algebraic variety

use std::{cmp::Ordering, fmt::Display, ops::Index, slice::SliceIndex};
use itertools::{EitherOrBoth, Itertools};
use num::Rational32;
use crate::{
    adic_valid,
    AdicApproximate, AdicInteger, AdicNumber,
    HasDigits, Prime, RationalAdicNumber, SignedAdicNumber, ZAdic,
};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An algebraic variety, a set of integer "roots" to an algebraic equation ([`zadic_variety`](crate::zadic_variety))
///
/// ```
/// # use adic::{ZAdic, AdicVariety};
/// let z = AdicVariety::new(5, vec![
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
pub struct AdicVariety<A>
where A: AdicNumber {
    p: Prime,
    elements: Vec<A>,
}


impl<A> AdicVariety<A>
where A: AdicNumber {

    /// Prime for this adic variety
    pub fn p(&self) -> Prime {
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
    /// let v = zadic_variety!(5, 3, [[1, 2, 3], [3, 2, 1], [2, 3, 1]]);
    /// assert_eq!(
    ///     vec![&zadic_approx!(5, 3, [1, 2, 3]), &zadic_approx!(5, 3, [2, 3, 1]), &zadic_approx!(5, 3, [3, 2, 1])],
    ///     v.roots().collect::<Vec<_>>()
    /// );
    /// ```
    pub fn roots(&self) -> impl Iterator<Item=&A> {
        self.elements.iter()
    }

    /// Iterator for the roots of this variety
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_variety};
    /// let v = zadic_variety!(5, 3, [[1, 2, 3], [3, 2, 1], [2, 3, 1]]);
    /// assert_eq!(
    ///     vec![zadic_approx!(5, 3, [1, 2, 3]), zadic_approx!(5, 3, [2, 3, 1]), zadic_approx!(5, 3, [3, 2, 1])],
    ///     v.into_roots().collect::<Vec<_>>()
    /// );
    /// ```
    pub fn into_roots(self) -> impl Iterator<Item=A> {
        self.elements.into_iter()
    }

    /// Slice of the roots of this variety
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_variety};
    /// let v = zadic_variety!(5, 3, [[1, 2, 3], [3, 2, 1], [2, 3, 1]]);
    /// assert_eq!(
    ///     &[zadic_approx!(5, 3, [1, 2, 3]), zadic_approx!(5, 3, [2, 3, 1]), zadic_approx!(5, 3, [3, 2, 1])],
    ///     v.root_slice()
    /// );
    /// assert_eq!(zadic_approx!(5, 3, [1, 2, 3]), v[0]);
    /// assert_eq!(zadic_approx!(5, 3, [2, 3, 1]), v[1]);
    /// assert_eq!(zadic_approx!(5, 3, [3, 2, 1]), v[2]);
    /// // v[3] <- panics like Vec
    /// ```
    pub fn root_slice(&self) -> &[A] {
        &self.elements
    }

    /// Get `Some(root)` at `index` or `None` if `index > v.num_roots()`
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_variety};
    /// let v = zadic_variety!(5, 3, [[1, 2, 3], [3, 2, 1], [2, 3, 1]]);
    /// assert_eq!(Some(&zadic_approx!(5, 3, [1, 2, 3])), v.get(0));
    /// assert_eq!(Some(&zadic_approx!(5, 3, [2, 3, 1])), v.get(1));
    /// assert_eq!(Some(&zadic_approx!(5, 3, [3, 2, 1])), v.get(2));
    /// assert_eq!(None, v.get(3));
    /// ```
    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[A]>>::Output>
    where I: SliceIndex<[A]> {
        self.elements.get(index)
    }

    /// Get `Some(root)` at `index` or `None` if `index > v.num_roots()`
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_variety};
    /// let mut v = zadic_variety!(5, 3, [[1, 2, 3], [3, 2, 1], [2, 3, 1]]);
    /// if let Some(a0) = v.get_mut(0) {
    ///     a0.pop_digit();
    /// }
    /// assert_eq!(Some(&zadic_approx!(5, 2, [1, 2])), v.get(0));
    /// assert_eq!(Some(&zadic_approx!(5, 3, [2, 3, 1])), v.get(1));
    /// assert_eq!(Some(&zadic_approx!(5, 3, [3, 2, 1])), v.get(2));
    /// ```
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[A]>>::Output>
    where I: SliceIndex<[A]> {
        self.elements.get_mut(index)
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
    pub fn new<P>(p: P, roots: Vec<A>) -> Self
    where P: Into<Prime>, A: AdicNumber + HasDigits + AdicApproximate {

        let p = p.into();

        adic_valid::validate_adic_same_p(p, &roots);
        let sorted_roots = sort_roots(roots).collect::<Vec<_>>();

        Self {
            p,
            elements: sorted_roots,
        }

    }

    /// Create an empty adic variety
    pub fn empty<P>(p: P) -> Self
    where P: Into<Prime> {
        Self {
            p: p.into(),
            elements: vec![],
        }
    }

    /// Create a `AdicVariety` from a Vec of `i32`s
    pub fn from_integer_roots<P>(p: P, roots: Vec<i32>) -> Self
    where P: Into<Prime>, A: SignedAdicNumber + AdicApproximate {
        let p = p.into();
        let roots = roots.into_iter().map(|root| A::from_i32(p, root)).collect();
        AdicVariety::new(p, roots)
    }

    /// Create a `AdicVariety<ZAdic>` from a Vec of `Rational32`s
    pub fn from_rational_roots<P>(p: P, roots: Vec<Rational32>) -> Self
    where P: Into<Prime>, A: RationalAdicNumber + AdicApproximate {
        let p = p.into();
        let roots = roots
            .into_iter()
            // Note: zapprox truncates fractions so watch out!!
            .map(|root| A::from_rational(p, root))
            .collect();
        AdicVariety::new(p, roots)
    }

    /// Approximation for this adic variety
    ///
    /// ```
    /// # use adic::{uadic, zadic_approx, AdicVariety};
    /// let av = AdicVariety::new(5, vec![uadic!(5, []), uadic!(5, [1]), uadic!(5, [1, 2, 3, 4])]);
    /// let zv = AdicVariety::new(5, vec![zadic_approx!(5, 2, []), zadic_approx!(5, 2, [1]), zadic_approx!(5, 2, [1, 2])]);
    /// assert_eq!(zv, av.approximation(2));
    /// ```
    pub fn approximation(&self, n: usize) -> AdicVariety<ZAdic>
    where A: AdicApproximate + AdicInteger {
        AdicVariety::new(self.p, self.roots().map(|r| r.approximation(n)).collect())
    }

    /// Approximation for this adic variety
    ///
    /// ```
    /// # use adic::{uadic, zadic_approx, AdicVariety};
    /// let av = AdicVariety::new(5, vec![uadic!(5, []), uadic!(5, [1]), uadic!(5, [1, 2, 3, 4])]);
    /// let zv = AdicVariety::new(5, vec![zadic_approx!(5, 2, []), zadic_approx!(5, 2, [1]), zadic_approx!(5, 2, [1, 2])]);
    /// assert_eq!(zv, av.into_approximation(2));
    /// ```
    pub fn into_approximation(self, n: usize) -> AdicVariety<ZAdic>
    where A: AdicApproximate + AdicInteger {
        AdicVariety::new(self.p, self.into_roots().map(|r| r.into_approximation(n)).collect())
    }

}


impl<A> Extend<A> for AdicVariety<A>
where A: AdicNumber + HasDigits + AdicApproximate {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {

        let tmp_vec = iter.into_iter().collect::<Vec<_>>();
        adic_valid::validate_adic_same_p(self.p(), &tmp_vec);
        let new_roots = sort_roots(
            self.elements.clone().into_iter().chain(tmp_vec)
        ).collect::<Vec<_>>();

        self.elements = new_roots;

    }
}

impl<A> Index<usize> for AdicVariety<A>
where A: AdicNumber {
    type Output = A;
    fn index(&self, index: usize) -> &Self::Output {
        self.root_slice().index(index)
    }
}

impl<A> Display for AdicVariety<A>
where A: AdicNumber + Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "variety(empty({}-adic))", self.p)
        } else {
            write!(f, "variety({})", self.elements.iter().map(ToString::to_string).join(", "))
        }
    }
}



fn sort_roots<A, I: IntoIterator<Item = A>>(roots: I) -> impl Iterator<Item = A>
where A: AdicNumber + HasDigits + AdicApproximate {
    roots.into_iter().sorted_by(|z1, z2| {
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
                Some(EitherOrBoth::Both(d1, d2)) => d1.cmp(&d2),
            }
        }
    })
}

/// Type definition for `ZAdicVariety`
pub type ZAdicVariety = AdicVariety<ZAdic>;


#[cfg(test)]
mod tests {
    use crate::{uadic, zadic_approx, zadic_exact, zadic_variety, AdicVariety, ZAdic};

    #[test]
    #[should_panic]
    fn bad_index_panics() {
        let empty_variety = AdicVariety::<ZAdic>::empty(5);
        let _a = empty_variety[0].clone();
    }

    #[test]
    fn roots_sorted() {

        let var = zadic_variety!(5, 2, [[3, 2], [3, 1], [4, 4], [0, 2], [4, 0]]);
        assert_eq!(zadic_variety!(5, 2, [[0, 2], [3, 1], [3, 2], [4, 0], [4, 4]]), var);
        assert_eq!(
            &[
                zadic_approx!(5, 2, [0, 2]), zadic_approx!(5, 2, [3, 1]), zadic_approx!(5, 2, [3, 2]),
                zadic_approx!(5, 2, [4, 0]), zadic_approx!(5, 2, [4, 4]),
            ],
            var.root_slice()
        );
        assert_eq!(var[0], zadic_approx!(5, 2, [0, 2]));
        assert_eq!(var[1], zadic_approx!(5, 2, [3, 1]));
        assert_eq!(var[2], zadic_approx!(5, 2, [3, 2]));
        assert_eq!(var[3], zadic_approx!(5, 2, [4, 0]));
        assert_eq!(var[4], zadic_approx!(5, 2, [4, 4]));
        assert_eq!(var.get(5), None);

        assert_eq!(
            &[zadic_approx!(5, 2, [2, 1]), zadic_approx!(5, 3, [2, 1, 0])],
            &AdicVariety::new(
                5, vec![zadic_approx!(5, 3, [2, 1, 0]), zadic_approx!(5, 2, [2, 1])]
            ).root_slice()
        );
        assert_eq!(
            &[zadic_exact!(uadic!(5, [2, 1])), zadic_approx!(5, 2, [2, 1])],
            &AdicVariety::new(
                5, vec![zadic_approx!(5, 2, [2, 1]), zadic_exact!(uadic!(5, [2, 1]))]
            ).root_slice()
        );

    }

    #[test]
    fn extend_roots() {
        let mut var = zadic_variety!(5, 2, [[3, 2], [3, 1]]);
        assert_eq!(
            &[zadic_approx!(5, 2, [3, 1]), zadic_approx!(5, 2, [3, 2])],
            &var.clone().root_slice()
        );
        var.extend([zadic_approx!(5, 2, [4, 4]), zadic_approx!(5, 2, [0, 2]), zadic_approx!(5, 2, [4, 0])]);
        assert_eq!(
            &[
                zadic_approx!(5, 2, [0, 2]), zadic_approx!(5, 2, [3, 1]), zadic_approx!(5, 2, [3, 2]),
                zadic_approx!(5, 2, [4, 0]), zadic_approx!(5, 2, [4, 4]),
            ],
            &var.root_slice()
        );
    }

    #[test]
    fn display() {

        let var = AdicVariety::<ZAdic>::empty(5);
        assert_eq!("variety(empty(5-adic))", var.to_string());

        let var = zadic_variety!(5, 6, [[0, 0, 0, 0, 0, 0], [1, 2, 3, 0, 1, 2]]);
        assert_eq!("variety(...000000._5, ...210321._5)", var.to_string());

        let var = AdicVariety::new(5, vec![zadic_exact!(uadic!(5, [1, 0, 3, 0, 0, 0])), zadic_approx!(5, 6, [1, 2, 3, 0, 1, 2])]);
        assert_eq!("variety(301._5, ...210321._5)", var.to_string());

    }

}
