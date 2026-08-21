use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
};
use itertools::Itertools;


#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An algebraic variety, often a set of "roots" to an algebraic equation
///
/// ```
/// # use adic::{Variety, EAdic, ZAdic};
/// let z = Variety::new(vec![
///     ZAdic::new_approx(5, 6, vec![0, 1, 2, 3]),
///     ZAdic::from(EAdic::new(5, vec![4, 3])),
/// ]);
/// assert_eq!(ZAdic::new_approx(5, 6, vec![0, 1, 2, 3]), z[0]);
/// assert_eq!(Some(&ZAdic::from(EAdic::new(5, vec![4, 3]))), z.get(1));
/// assert_eq!(None, z.get(2));
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
///
/// Note that the roots are not sorted in any way, and multiplicity/degeneracy is allowed.
/// If sorting is desired, the [`adic_sort`](Variety::adic_sort) and [`adic_sorted`](Variety::adic_sorted)
///  methods will sort adic numbers by digit.
pub struct Variety<T> {
    pub (super) elements: Vec<T>,
}


impl<T> Variety<T> {

    /// Create a variety with the given roots/solutions
    pub fn new(roots: Vec<T>) -> Self {
        Self {
            elements: roots,
        }
    }

    /// Number of roots in variety
    pub fn num_roots(&self) -> usize {
        self.elements.len()
    }

    /// Iterator reference for the roots of this variety
    ///
    /// ```
    /// # use adic::{Variety, ZAdic};
    /// let v = Variety::<ZAdic>::primed_from_roots(5, [86, 38, 42]);
    /// let v = v.into_approximation(3).adic_sorted();
    /// assert_eq!(
    ///     vec![
    ///         &ZAdic::new_approx(5, 3, vec![1, 2, 3]),
    ///         &ZAdic::new_approx(5, 3, vec![2, 3, 1]),
    ///         &ZAdic::new_approx(5, 3, vec![3, 2, 1]),
    ///     ],
    ///     v.roots().collect::<Vec<_>>()
    /// );
    /// ```
    pub fn roots(&self) -> impl Iterator<Item=&T> {
        self.elements.iter()
    }

    /// Iterator for the roots of this variety
    ///
    /// ```
    /// # use adic::{Variety, ZAdic};
    /// let v = Variety::<ZAdic>::primed_from_roots(5, [86, 38, 42]);
    /// let v = v.into_approximation(3).adic_sorted();
    /// assert_eq!(
    ///     vec![
    ///         ZAdic::new_approx(5, 3, vec![1, 2, 3]),
    ///         ZAdic::new_approx(5, 3, vec![2, 3, 1]),
    ///         ZAdic::new_approx(5, 3, vec![3, 2, 1]),
    ///     ],
    ///     v.into_roots().collect::<Vec<_>>()
    /// );
    /// ```
    pub fn into_roots(self) -> impl Iterator<Item=T> {
        self.elements.into_iter()
    }

    /// Do no roots exist
    ///
    /// ```
    /// # use adic::{Variety, ZAdic};
    /// let v = Variety::new(vec![
    ///     ZAdic::new_approx(5, 3, vec![1, 2, 3]),
    ///     ZAdic::new_approx(5, 3, vec![3, 2, 1]),
    /// ]);
    /// assert!(!v.is_empty());
    /// let v = Variety::<ZAdic>::new(vec![]);
    /// assert!(v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Create an empty adic variety
    pub fn empty() -> Self {
        Self {
            elements: vec![],
        }
    }

    /// Returns the number of roots in the variety
    pub const fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns a reference to an element or `None` if out of bounds
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.elements.get(idx)
    }

}


impl<T> PartialEq for Variety<T>
where T: Eq + Hash {
    fn eq(&self, other: &Self) -> bool {
        let (mut hashmap0, mut hashmap1) = (HashMap::new(), HashMap::new());
        for root in self.roots() {
            *hashmap0.entry(root).or_insert(0) += 1;
        }
        for root in other.roots() {
            *hashmap1.entry(root).or_insert(0) += 1;
        }
        hashmap0 == hashmap1
    }
}
impl<T> Eq for Variety<T>
where T: Eq + Hash { }


impl<T> std::ops::Index<usize> for Variety<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}

impl<T> Extend<T> for Variety<T> {
    fn extend<U: IntoIterator<Item = T>>(&mut self, iter: U) {
        self.elements.extend(iter);
    }
}

impl<T> Display for Variety<T>
where T: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "variety(empty)")
        } else {
            write!(f, "variety({})", self.elements.iter().map(ToString::to_string).join(", "))
        }
    }
}
