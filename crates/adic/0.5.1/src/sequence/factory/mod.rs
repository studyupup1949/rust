//! Generate sequences

mod constant_sequence;
mod power_sequence;
mod recurrent;

use std::ops::{Mul, Div};
use crate::local_num::{LocalOne, LocalZero};
use constant_sequence::ConstantSequence;
use power_sequence::PowerSequence;
use super::Sequence;


/// Create a constant sequence
/// ```
/// # use adic::sequence::Sequence;
/// let t = adic::sequence::factory::constant(7);
/// assert_eq!(vec![7, 7, 7, 7], t.truncation(4));
/// ```
pub fn constant<T>(term: T) -> impl Clone + Sequence<Term=T>
where T: Clone {
    ConstantSequence::new(term)
}

/// Create a linear sequence starting at 0 and increasing by term
pub fn linear<T>(term: T) -> impl Clone + Sequence<Term=T>
where T: Clone + LocalZero {
    let zero = term.local_zero();
    constant(term).term_scan(zero, |ongoing, t| {
        let out = ongoing.clone();
        *ongoing = ongoing.clone() + t;
        out
    })
}

/// Create a power series, `coefficient * (t^0 + t^(1 * power) + t^(2 * power) + ...`
pub fn power<T>(coefficient: T, term: T, power: u32) -> impl Clone + Sequence<Term=T>
where T: Clone + LocalOne {
    PowerSequence::new(coefficient, term, power)
}

/// Creates a geometric progression sequence
/// ```
/// # use adic::sequence::Sequence;
/// let t = adic::sequence::factory::geometric_progression(2);
/// assert_eq!(vec![1, 2, 4, 8, 16, 32, 64, 128, 256, 512], t.truncation(10));
/// ```
pub fn geometric_progression<T>(term: T) -> impl Clone + Sequence<Term=T>
where T: Clone + LocalOne {
    PowerSequence::new(term.local_one(), term, 1)
}

/// Create a sequence for the exponential series terms, x^n / n!
pub fn exponential_progression<T>(x: T) -> impl Clone + Sequence<Term=T>
where T: Clone + LocalZero + LocalOne + Mul<Output = T> + Div<Output = T> {
    let zero = x.local_zero();
    let one = x.local_one();
    let denom = linear(one.clone()).skip(1);
    let numer = constant(x);
    numer.term_compose(denom, (zero.clone(), zero.clone()), |n, d| n / d).term_scan(one, |acc, t| {
        let out = acc.clone();
        *acc = acc.clone() * t;
        out
    })
}

/// Creates a new Lucas [`Sequence`]
pub fn lucas(initial_values: (u32, u32)) -> impl Clone + Sequence<Term=u32> {
    recurrent::Lucas::new(initial_values)
}

/// Creates the Fibonacci [`Sequence`]
pub fn fibonacci() -> impl Clone + Sequence<Term=u32> {
    recurrent::Lucas::fibonacci()
}

/// Creates a new Collatz [`Sequence`] starting at the given element
pub fn collatz(starting_element: u32) -> impl Clone + Sequence<Term=u32> {
    recurrent::Collatz::new(starting_element)
}

/// Creates a Thue-Morse [`Sequence`] given a starting element of 0 or 1
pub fn thue_morse(starting_element: u32) -> impl Clone + Sequence<Term=u32> {
    recurrent::ThueMorse::new(starting_element)
}



#[cfg(test)]
mod test;
