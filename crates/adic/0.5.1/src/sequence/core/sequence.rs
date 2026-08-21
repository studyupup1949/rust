use std::{
    ops::{Add, Mul, Sub},
    rc::Rc,
};
use crate::{
    divisible::Prime,
    local_num::{LocalOne, LocalZero},
    traits::AdicPrimitive,
};
use super::derived::{
    CarrySequence, EnumeratedSequence, FoilMulSequence, MappedSequence,
    ScannedSequence, SkippedSequence, TermAddSequence, TermComposedSequence, TermMulSequence,
};


/// A general sequence of terms
///
/// Can be combined with other `Sequences` with e.g. `term_add` or `foil_mul`.
/// Designed to do iterator-like tasks, but `Sequence` is a collection, not an iterator.
///
/// Note: This trait will likely change in `adic 0.6`.
pub trait Sequence {

    // Required

    /// Type for the terms of the `Sequence`
    type Term;

    /// Returns an iterator of Sequence terms
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_>;

    /// How many terms are in the `Sequence` or `None` if it is infinite
    ///
    /// Note: do **not** expect this to be an exact measure of the number of terms.
    /// Do not even expect it to match `None` when infinite, although it **should**.
    fn approx_num_terms(&self) -> Option<usize>;


    // Sequence properties

    /// Is a finite sequence
    fn is_finite_sequence(&self) -> bool {
        self.approx_num_terms().is_some()
    }

    /// Does the `Sequence` contain no terms
    fn is_empty(&self) -> bool {
        self.approx_num_terms().is_some_and(|n| n == 0)
    }


    // Term retrieval

    /// Returns the term at the index
    fn term(&self, n: usize) -> Option<Self::Term>
    where Self: Sized {
        self.terms().nth(n)
    }

    /// First term; None if `Sequence` is empty
    fn first_term(&self) -> Option<Self::Term> {
        self.terms().next()
    }

    /// Retrieve a local zero from first term; None if `Sequence` is empty
    fn term_local_zero(&self) -> Option<Self::Term>
    where Self::Term: LocalZero {
        self.first_term().map(|t| t.local_zero())
    }

    /// Retrieve a local zero from first term; None if `Sequence` is empty
    fn term_local_one(&self) -> Option<Self::Term>
    where Self::Term: LocalOne {
        self.first_term().map(|t| t.local_one())
    }

    /// Retrieve a prime from first term; None if `Sequence` is empty
    fn term_local_prime(&self) -> Option<Prime>
    where Self::Term: AdicPrimitive {
        self.first_term().map(|t| t.p())
    }


    // Derived Sequences

    /// Truncates a Sequence and returns a Vec
    fn truncation(&self, len: usize) -> Vec<Self::Term> {
        self.terms().take(len).collect::<Vec<_>>()
    }

    /// Enumerate a Sequence
    fn enumerate(self) -> EnumeratedSequence<Self>
    where Self: Sized {
        EnumeratedSequence::new(self)
    }

    /// Skip `n` terms of a Sequence
    fn skip(self, n: usize) -> SkippedSequence<Self>
    where Self: Sized {
        SkippedSequence::new(self, n)
    }

    /// Map a Sequence to another with the given `op`, term by term
    fn term_map<F, U>(self, op: F) -> MappedSequence<Self, F, U>
    where Self: Sized, F: Fn(Self::Term) -> U {
        MappedSequence::new(self, op)
    }

    /// Scans a Sequence, mutating `initial` and returning it, term by term
    fn term_scan<St, F, U>(self, initial: St, op: F) -> ScannedSequence<Self, St, F, U>
    where St: Clone, Self: Sized, F: Fn(&mut St, Self::Term) -> U {
        ScannedSequence::new(self, initial, op)
    }

    /// Add two Sequences, term by term
    fn term_add<S>(self, other: S) -> TermAddSequence<Self, S>
    where Self: Sized, S: Sequence<Term=Self::Term>, Self::Term: Clone + Add<Output=Self::Term> {
        TermAddSequence::new(self, other)
    }

    /// Multiply two Sequences, term by term
    ///
    /// If `trailing_ones` is `true`, the `Sequence` continues with `1` after it ends.
    /// If `trailing_ones` is `false`, the `Sequence` terminates with `0` after it ends.
    fn term_mul<S>(self, other: S, trailing_ones: bool) -> TermMulSequence<Self, S>
    where Self: Sized, S: Sequence<Term=Self::Term>, Self::Term: Clone + Mul<Output=Self::Term> {
        TermMulSequence::new(self, other, trailing_ones)
    }

    /// Compose two Sequences with the given `op`, term by term
    ///
    /// `default` gives the term that should be used after one of the sequences terminates.
    fn term_compose<S, F, U>(self, other: S, default: (Self::Term, S::Term), op: F) -> TermComposedSequence<Self, S, F, U>
    where Self::Term: Clone, Self: Sized, S: Sequence, S::Term: Clone, F: Fn(Self::Term, S::Term) -> U {
        TermComposedSequence::new(self, other, default, op)
    }

    /// Multiply two Sequences by FOIL,
    ///  i.e. `(a, b, c...).foil_mul(d, e, f...) = (ad, ae + bd, af + be + cd...)`
    fn foil_mul<S>(self, other: S) -> FoilMulSequence<Self, S>
    where Self: Sized, Self::Term: Clone + Mul<Output = Self::Term> + Add<Output = Self::Term>, S: Sequence<Term=Self::Term> {
        FoilMulSequence::new(self, other)
    }

    /// Rectify Sequence terms to be `0 <= term < modulus`, carrying remainders
    fn carry_with(self, modulus: Self::Term) -> CarrySequence<Self>
    where Self: Sized, Self::Term: LocalZero + LocalOne + PartialOrd + Add<Output=Self::Term> + Sub<Output=Self::Term> {
        CarrySequence::new(self, modulus)
    }

}

impl<T> Sequence for Vec<T>
where T: Clone {
    type Term = T;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        Box::new(self.iter().cloned())
    }
    fn approx_num_terms(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<F, T> Sequence for F
where F: Fn(usize) -> T {
    type Term = T;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        Box::new((0..).map(self))
    }
    fn approx_num_terms(&self) -> Option<usize> {
        None
    }
}

impl<T> Sequence for Box<dyn Sequence<Term = T> + '_> {
    type Term = T;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        self.as_ref().terms()
    }
    fn approx_num_terms(&self) -> Option<usize> {
        self.as_ref().approx_num_terms()
    }
}

impl<T> Sequence for Rc<dyn Sequence<Term = T> + '_> {
    type Term = T;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        self.as_ref().terms()
    }
    fn approx_num_terms(&self) -> Option<usize> {
        self.as_ref().approx_num_terms()
    }
}



#[cfg(test)]
mod test {

    use num::Rational32;
    use crate::combinatorics::factorial;
    use super::Sequence;

    #[test]
    fn truncation() {

        let s = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(s.truncation(3), vec![1, 2, 3]);
        assert_eq!(s.truncation(10), s);

        let s = |i| i*2;
        assert_eq!(s.truncation(3), vec![0, 2, 4]);
        assert_eq!(s.truncation(0), vec![]);

        let s = Vec::<i32>::new();
        assert_eq!(s.truncation(3), vec![]);

    }

    #[test]
    fn finite() {
        let s = vec![1, -2, 1];

        assert_eq!(s.terms().collect::<Vec<_>>(), vec![1, -2, 1]);
        assert!(s.is_finite_sequence());

        assert!(!(|_| 3).is_finite_sequence());
    }

    #[test]
    fn constant() {
        let function = |_| 3;

        assert_eq!(function.terms().take(3).collect::<Vec<_>>(), vec![3, 3, 3]);
        assert_eq!(function.terms().take(0).collect::<Vec<_>>(), vec![]);
        assert_eq!(function.terms().take(5).collect::<Vec<_>>(), vec![3, 3, 3, 3, 3]);
    }

    #[test]
    fn linear() {
        let function = |x| 2 * x;

        assert_eq!(function.terms().take(3).collect::<Vec<_>>(), vec![0, 2, 4]);
        assert_eq!(function.terms().take(0).collect::<Vec<_>>(), vec![]);
        assert_eq!(function.terms().take(0).collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn geometric_progression() {
        let function = |x| 2u32.pow(u32::try_from(x).unwrap());

        assert_eq!(function.terms().take(5).collect::<Vec<_>>(), vec![1, 2, 4, 8, 16]);
        assert_eq!(function.terms().take(5).collect::<Vec<_>>(), vec![1, 2, 4, 8, 16]);
    }

    #[test]
    fn exponential_progression() {
        let function = |x| 2u32.pow(u32::try_from(x).unwrap()) / factorial::<u32>(u32::try_from(x).unwrap());

        assert_eq!(function.terms().take(5).collect::<Vec<_>>(), vec![1, 2, 2, 1, 0]);
        assert_eq!(function.terms().take(5).collect::<Vec<_>>(), vec![1, 2, 2, 1, 0]);

        let base = Rational32::new(2, 1);
        let function = |x| base.pow(i32::try_from(x).unwrap()) / factorial::<Rational32>(u32::try_from(x).unwrap());

        assert_eq!(
            function.terms().take(5).collect::<Vec<_>>(),
            vec![Rational32::new(1, 1), Rational32::new(2, 1), Rational32::new(2, 1), Rational32::new(4, 3), Rational32::new(2, 3)]
        );
    }

    #[test]
    fn compose() {
        let a = vec![0, 1, 2, 3];
        let b = vec![0, 2, 4];

        let c = a.clone().term_compose(b.clone(), (0, 0), |at, bt| (at + bt) / 2);
        assert_eq!(c.terms().collect::<Vec<_>>(), vec![0, 1, 3, 1]);

        // Addition assumes **zero** if unequal length
        let c = a.clone().term_add(b.clone());
        assert_eq!(c.terms().collect::<Vec<_>>(), vec![0, 3, 6, 3]);

        // Multiplication can assume **zero** if unequal length and **truncate**
        let c = a.clone().term_mul(b.clone(), false);
        assert_eq!(c.terms().collect::<Vec<_>>(), vec![0, 2, 8]);

        // ..or assume **one** if unequal length
        let c = a.clone().term_mul(b.clone(), true);
        assert_eq!(c.terms().collect::<Vec<_>>(), vec![0, 2, 8, 3]);

        // ...but FOIL multiplication assumes **zero** if unequal length!
        let c = a.clone().foil_mul(b.clone());
        assert_eq!(c.terms().collect::<Vec<_>>(), vec![0, 0, 2, 8, 14, 12]);

        let c = a.clone().carry_with(2);
        assert_eq!(c.terms().collect::<Vec<_>>(), vec![0, 1, 0, 0, 0, 1]);

    }

}
