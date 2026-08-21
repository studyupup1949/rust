//! Map and compose [`Sequences`](Sequence)

use std::{
    cmp::PartialOrd,
    iter::repeat,
    ops::{Add, Mul, Sub},
};
use itertools::{EitherOrBoth, Itertools};
use crate::local_num::{LocalOne, LocalZero};
use super::Sequence;


#[derive(Debug, Clone)]
/// Enumerate a [`Sequence`]
pub struct EnumeratedSequence<S>
where S: Sequence {
    sequence: S,
}

impl<S> EnumeratedSequence<S>
where S: Sequence {
    /// Constructor
    pub fn new(sequence: S) -> Self {
        Self {
            sequence,
        }
    }
}

impl<S> Sequence for EnumeratedSequence<S>
where S: Sequence {
    type Term = (usize, S::Term);
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        Box::new(self.sequence.terms().enumerate())
    }
    fn approx_num_terms(&self) -> Option<usize> {
        self.sequence.approx_num_terms()
    }
}


#[derive(Debug, Clone)]
/// Skip some terms of a [`Sequence`]
pub struct SkippedSequence<S>
where S: Sequence {
    sequence: S,
    n: usize,
}

impl<S> SkippedSequence<S>
where S: Sequence {
    /// Constructor
    pub fn new(sequence: S, n: usize) -> Self {
        Self {
            sequence,
            n,
        }
    }
}

impl<S> Sequence for SkippedSequence<S>
where S: Sequence {
    type Term = S::Term;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        Box::new(self.sequence.terms().skip(self.n))
    }
    fn approx_num_terms(&self) -> Option<usize> {
        match self.sequence.approx_num_terms() {
            Some(n) if n > self.n => Some(n-self.n),
            Some(_) => Some(0),
            None => None,
        }
    }
}


#[derive(Debug, Clone)]
/// Map a [`Sequence`], term by term
pub struct MappedSequence<S, F, U>
where S: Sequence, F: Fn(S::Term) -> U {
    sequence: S,
    op: F,
}

impl<S, F, U> MappedSequence<S, F, U>
where S: Sequence, F: Fn(S::Term) -> U {
    /// Constructor
    pub fn new(sequence: S, op: F) -> Self {
        Self {
            sequence,
            op,
        }
    }
}

impl<S, F, U> Sequence for MappedSequence<S, F, U>
where S: Sequence, F: Fn(S::Term) -> U {
    type Term = U;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        Box::new(self.sequence.terms().map(move |t| (self.op)(t)))
    }
    fn approx_num_terms(&self) -> Option<usize> {
        self.sequence.approx_num_terms()
    }
}


#[derive(Debug, Clone)]
/// Scan a [`Sequence`], term by term, using op to change a state value for each term
pub struct ScannedSequence<S, St, F, U>
where S: Sequence, St: Clone, F: Fn(&mut St, S::Term) -> U {
    sequence: S,
    initial: St,
    op: F,
}

impl<S, St, F, U> ScannedSequence<S, St, F, U>
where S: Sequence, St: Clone, F: Fn(&mut St, S::Term) -> U {
    /// Constructor
    pub fn new(sequence: S, initial: St, op: F) -> Self {
        Self {
            sequence,
            initial,
            op,
        }
    }
}

impl<S, St, F, U> Sequence for ScannedSequence<S, St, F, U>
where S: Sequence, St: Clone, F: Fn(&mut St, S::Term) -> U {
    type Term = U;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        let mut current = self.initial.clone();
        Box::new(self.sequence.terms().map(move |t| (self.op)(&mut current, t)))
    }
    fn approx_num_terms(&self) -> Option<usize> {
        self.sequence.approx_num_terms()
    }
}


#[derive(Debug, Clone)]
/// Add two [`Sequences`](Sequence), term by term, assuming `zero` if one `Sequence` ends early
pub struct TermAddSequence<A, B>
where A: Sequence, B: Sequence<Term=A::Term>, A::Term: Clone + Add<Output=A::Term> {
    sequence1: A,
    sequence2: B,
}

impl<A, B> TermAddSequence<A, B>
where A: Sequence, B: Sequence<Term=A::Term>, A::Term: Clone + Add<Output=A::Term> {
    /// Constructor
    pub fn new(sequence1: A, sequence2: B) -> Self {
        Self {
            sequence1,
            sequence2,
        }
    }
}

impl<A, B> Sequence for TermAddSequence<A, B>
where A: Sequence, B: Sequence<Term=A::Term>, A::Term: Clone + Add<Output=A::Term> {
    type Term = A::Term;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        Box::new(self.sequence1.terms().zip_longest(self.sequence2.terms()).map(
            |zipped| zipped.reduce(|a, b| a + b)
        ))
    }
    fn approx_num_terms(&self) -> Option<usize> {
        if let (Some(n1), Some(n2)) = (self.sequence1.approx_num_terms(), self.sequence2.approx_num_terms()) {
            Some(std::cmp::max(n1, n2))
        } else {
            None
        }
    }
}


#[derive(Debug, Clone)]
/// Mul two [`Sequences`](Sequence), term by term, assuming `one` if one `Sequence` ends early
pub struct TermMulSequence<A, B>
where A: Sequence, B: Sequence<Term=A::Term>, A::Term: Clone + Mul<Output=A::Term> {
    sequence1: A,
    sequence2: B,
    trailing_ones: bool,
}

impl<A, B> TermMulSequence<A, B>
where A: Sequence, B: Sequence<Term=A::Term>, A::Term: Clone + Mul<Output=A::Term> {
    /// Constructor
    pub fn new(sequence1: A, sequence2: B, trailing_ones: bool) -> Self {
        Self {
            sequence1,
            sequence2,
            trailing_ones,
        }
    }
}

impl<A, B> Sequence for TermMulSequence<A, B>
where A: Sequence, B: Sequence<Term=A::Term>, A::Term: Clone + Mul<Output=A::Term> {
    type Term = A::Term;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        if self.trailing_ones {
            Box::new(self.sequence1.terms().zip_longest(self.sequence2.terms()).map(
                |zipped| zipped.reduce(|a, b| a * b)
            ))
        } else {
            Box::new(self.sequence1.terms().zip_longest(self.sequence2.terms()).map_while(
                |zipped| zipped.both().map(|(a, b)| a * b)
            ))
        }
    }
    fn approx_num_terms(&self) -> Option<usize> {
        if self.trailing_ones {
            None
        } else {
            match (self.sequence1.approx_num_terms(), self.sequence2.approx_num_terms()) {
                (Some(n1), Some(n2)) => Some(std::cmp::min(n1, n2)),
                (Some(n1), None) => Some(n1),
                (None, Some(n2)) => Some(n2),
                (None, None) => None,
            }
        }
    }
}


#[derive(Debug, Clone)]
/// A [`Sequence`] that composes two `Sequences` with the given `op`, term by term
pub struct TermComposedSequence<A, B, F, U>
where A: Sequence, A::Term: Clone, B: Sequence, B::Term: Clone, F: Fn(A::Term, B::Term) -> U {
    sequence1: A,
    sequence2: B,
    default: (A::Term, B::Term),
    op: F,
}

impl<A, B, F, U> TermComposedSequence<A, B, F, U>
where A: Sequence, A::Term: Clone, B: Sequence, B::Term: Clone, F: Fn(A::Term, B::Term) -> U {
    /// Constructor
    pub fn new(sequence1: A, sequence2: B, default: (A::Term, B::Term), op: F) -> Self {
        Self {
            sequence1,
            sequence2,
            default,
            op,
        }
    }
}

impl<A, B, F, U> Sequence for TermComposedSequence<A, B, F, U>
where A: Sequence, A::Term: Clone, B: Sequence, B::Term: Clone, F: Fn(A::Term, B::Term) -> U {
    type Term = U;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        Box::new(self.sequence1.terms().zip_longest(self.sequence2.terms()).map(|zipped| {
            let (a, b) = zipped.or(self.default.0.clone(), self.default.1.clone());
            (self.op)(a, b)
        }))
    }
    fn approx_num_terms(&self) -> Option<usize> {
        if let (Some(n1), Some(n2)) = (self.sequence1.approx_num_terms(), self.sequence2.approx_num_terms()) {
            Some(std::cmp::max(n1, n2))
        } else {
            None
        }
    }
}


#[derive(Debug, Clone)]
/// A [`Sequence`] that composes two `Sequences` by performing FOIL-style multiplication
pub struct FoilMulSequence<A, B>
where A: Sequence,
B: Sequence<Term=A::Term>,
A::Term: Clone + Add<Output=A::Term> + Mul<Output=A::Term> {
    sequence1: A,
    sequence2: B,
}

impl<A, B> FoilMulSequence<A, B>
where A: Sequence,
B: Sequence<Term=A::Term>,
A::Term: Clone + Add<Output=A::Term> + Mul<Output=A::Term> {
    /// Constructor
    pub fn new(a: A, b: B) -> Self {
        Self {
            sequence1: a,
            sequence2: b,
        }
    }
}

impl<A, B> Sequence for FoilMulSequence<A, B>
where A: Sequence,
B: Sequence<Term=A::Term>,
A::Term: Clone + Add<Output=A::Term> + Mul<Output=A::Term> {
    type Term = A::Term;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        let (mut termvec1, mut termvec2) = (vec![], vec![]);
        let terms = self.sequence1.terms()
            .zip_longest(self.sequence2.terms())
            .map(Some)
            .chain(repeat(None))
            .enumerate()
            .map_while(move |(out_idx, zipped)| {

            let (l, r) = zipped.map_or((None, None), EitherOrBoth::left_and_right);
            if let Some(a) = l {
                termvec1.push(a);
            }
            if let Some(b) = r {
                termvec2.push(b);
            }
            foil_idx(out_idx, &termvec1, &termvec2)

        });
        Box::new(terms)
    }
    fn approx_num_terms(&self) -> Option<usize> {
        if let (Some(n1), Some(n2)) = (self.sequence1.approx_num_terms(), self.sequence2.approx_num_terms()) {
            if n1 == 0 && n2 == 0 {
                Some(0)
            } else {
                Some(n1 + n2 - 1)
            }
        } else {
            None
        }
    }
}

fn foil_idx<T>(out_idx: usize, a: &[T], b: &[T]) -> Option<T>
where T: Clone + Add<Output=T> + Mul<Output=T> {
    // Doing foil:
    // - store vectors of the terms
    // - reverse second vector
    // - add multiplied terms together
    let coeffs1 = a.to_owned();
    let mut rev_coeffs2 = b.to_owned();
    rev_coeffs2.reverse();
    let skip2 = out_idx + 1 - rev_coeffs2.len();
    coeffs1.into_iter().skip(skip2).zip(rev_coeffs2).map(|(a, b)| a * b).reduce(|acc, t| acc + t)
}


#[derive(Debug, Clone)]
/// A [`Sequence`] that rectifies a given `Sequence` by performing carrying by a given `modulus`
pub struct CarrySequence<S>
where S: Sequence, S::Term: LocalZero + LocalOne + PartialOrd + Add<Output=S::Term> + Sub<Output=S::Term> {
    sequence: S,
    modulus: S::Term,
}

impl<S> CarrySequence<S>
where S: Sequence, S::Term: LocalZero + LocalOne + PartialOrd + Add<Output=S::Term> + Sub<Output=S::Term> {
    /// Constructor
    pub fn new(sequence: S, modulus: S::Term) -> Self {
        Self {
            sequence,
            modulus,
        }
    }
}

impl<S> Sequence for CarrySequence<S>
where S: Sequence, S::Term: Clone + LocalZero + LocalOne + PartialOrd + Add<Output=S::Term> + Sub<Output=S::Term> {
    type Term = S::Term;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        let zero = self.modulus.local_zero();
        let one = self.modulus.local_one();
        let mut carry = zero.clone();
        let modulus = self.modulus.clone();
        let terms = self.sequence.terms()
            .map(Some)
            .chain(repeat(None))
            .map_while(move |t| {

            if t.is_none() && carry.is_local_zero() {
                return None;
            }

            let mut t = t.unwrap_or(zero.clone()) + carry.clone();
            carry = zero.clone();
            while t >= modulus {
                carry = carry.clone() + one.clone();
                t = t - modulus.clone();
            }
            while t < zero {
                carry = carry.clone() - one.clone();
                t = t + modulus.clone();
            }
            Some(t)

        });
        Box::new(terms)
    }
    fn approx_num_terms(&self) -> Option<usize> {
        // TODO: Actually this is just a lower bound; try to find a way to efficiently compute this
        //  or else make `approx_num_terms` return a three-state (or more) structure instead of Option<usize>
        if self.modulus.is_local_zero() || self.modulus.is_local_one() {
            None
        } else {
            self.sequence.approx_num_terms()
        }
    }
}



#[cfg(test)]
mod test {

    use super::Sequence;

    #[test]
    fn non_op() {
        let s = vec!['a', 'b', 'c', 'd'];
        let en = s.clone().enumerate();
        assert!(en.is_finite_sequence());
        assert_eq!(en.terms().collect::<Vec<_>>(), vec![(0, 'a'), (1, 'b'), (2, 'c'), (3, 'd')]);
        let sk = s.clone().skip(2);
        assert!(sk.is_finite_sequence());
        assert_eq!(sk.terms().collect::<Vec<_>>(), vec!['c', 'd']);
        let mp = s.clone().term_map(|c| c.to_ascii_uppercase());
        assert!(mp.is_finite_sequence());
        assert_eq!(mp.terms().collect::<Vec<_>>(), vec!['A', 'B', 'C', 'D']);
        let sn = s.clone().term_scan("letter".to_string(), |acc, c| {
            acc.push(c);
            acc.clone()
        });
        assert!(sn.is_finite_sequence());
        assert_eq!(
            sn.terms().collect::<Vec<_>>(),
            vec!["lettera", "letterab", "letterabc", "letterabcd"]
        );
    }

    #[test]
    fn ops() {
        let s0 = |n| n * n;
        let s1 = vec![1, 5, 25];
        let add = s0.term_add(s1.clone());
        assert!(!add.is_finite_sequence());
        assert_eq!(add.truncation(6), vec![1, 6, 29, 9, 16, 25]);
        let mul_no_trailing = s0.term_mul(s1.clone(), false);
        assert!(mul_no_trailing.is_finite_sequence());
        assert_eq!(mul_no_trailing.terms().collect::<Vec<_>>(), vec![0, 5, 100]);
        let mul_trailing = s0.term_mul(s1.clone(), true);
        assert!(!mul_trailing.is_finite_sequence());
        assert_eq!(mul_trailing.truncation(6), vec![0, 5, 100, 9, 16, 25]);
        let foil_mul = s0.foil_mul(s1.clone());
        assert!(!foil_mul.is_finite_sequence());
        assert_eq!(foil_mul.truncation(6), vec![0, 1, 9, 54, 161, 330]);
        let carry0 = s0.carry_with(10);
        assert!(!carry0.is_finite_sequence());
        assert_eq!(carry0.truncation(6), vec![0, 1, 4, 9, 6, 6]);
        let carry1 = s1.carry_with(10);
        assert!(carry1.is_finite_sequence());
        assert_eq!(carry1.terms().collect::<Vec<_>>(), vec![1, 5, 5, 2]);
    }

}
