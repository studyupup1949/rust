//! Recurrent [`Sequences`](Sequence)

use std::iter::repeat_with;
use super::Sequence;


#[derive(Debug, Clone)]
/// Lucas [`Sequence`]
pub struct Lucas {
    initial_values: (u32, u32),
}

impl Lucas {
    /// Creates a new Lucas [`Sequence`]
    pub fn new(initial_values: (u32, u32)) -> Self {
        Self { initial_values }
    }

    /// Creates the Fibonacci [`Sequence`]
    pub fn fibonacci() -> Self {
        Self { initial_values: (0, 1) }
    }
}

impl Sequence for Lucas {
    type Term = u32;

    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        let mut two_terms = self.initial_values;
        Box::new(repeat_with(move || {
            let out = two_terms.0;
            let sum = two_terms.0 + two_terms.1;
            two_terms.0 = two_terms.1;
            two_terms.1 = sum;

            out
        }))
    }

    fn approx_num_terms(&self) -> Option<usize> {
        None
    }
}

#[derive(Debug, Clone)]
/// Collatz [`Sequence`]
pub struct Collatz {
    /// Starting element for Collatz procedure
    starting_element: u32,
}

impl Collatz {
    /// Creates a new Collatz [`Sequence`] starting at the given element
    pub fn new(starting_element: u32)  -> Self {
        Self { starting_element }
    }
}

impl Sequence for Collatz {
    type Term = u32;

    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        let mut last = self.starting_element;
        Box::new(std::iter::once(self.starting_element).chain(repeat_with(move || {
            last = if last.is_multiple_of(2) {
                last / 2
            } else {
                3 * last + 1
            };

            last
        })))
    }

    fn approx_num_terms(&self) -> Option<usize> {
        None
    }
}

#[derive(Debug, Clone)]
/// Thue-Morse [`Sequence`]
///
///```
/// # use adic::sequence::{factory, Sequence};
/// let t0 = factory::thue_morse(0);
/// assert_eq!(vec![0, 1, 1, 0, 1, 0, 0, 1, 1, 0], t0.truncation(10));
/// let t1 = factory::thue_morse(1);
/// assert_eq!(vec![1, 0, 0, 1, 0, 1, 1, 0, 0, 1], t1.truncation(10));
///
/// let t_added = t0.clone().term_add(t1.clone());
/// assert_eq!(vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1], t_added.truncation(10));
///
/// let t_mult = t0.clone().term_mul(t1.clone(), false);
/// assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0], t_mult.truncation(10));
/// let t_mult = t0.term_mul(t1, true);
/// assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0], t_mult.truncation(10));
///```
pub struct ThueMorse {
    /// Starting element for Thue-Morse sequence
    starting_element: u32,
}

impl ThueMorse {
    /// Creates a Thue-Morse [`Sequence`] given a starting element of 0 or 1
    pub fn new(starting_element: u32) -> Self {
        assert!([0, 1].contains(&starting_element), "Thue-Morse sequence starts with 0 or 1");
        Self { starting_element }
    }
}

impl Sequence for ThueMorse {
    type Term = u32;

    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        // NOTE: There may be more performant ways of doing this (https://en.wikipedia.org/wiki/Thue%E2%80%93Morse_sequence)

        let starting_element = self.starting_element;

        let mut cached_sequence = vec![starting_element];

        Box::new(std::iter::once(starting_element).chain((1..).map(move |index| {
            let new_term = if index % 2 == 0 {
                cached_sequence[index/2]
            } else {
                1 - cached_sequence[index/2]
            };
            cached_sequence.push(new_term);
            new_term
        })))
    }

    fn approx_num_terms(&self) -> Option<usize> {
        None
    }
}
