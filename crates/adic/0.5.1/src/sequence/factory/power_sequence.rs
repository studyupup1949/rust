#![allow(dead_code)]

use std::iter::{once, repeat_with};
use num::Zero;

use crate::{
    local_num::LocalOne,
    normed::{UltraNormed, Valuation, ValuationRing},
};

use super::Sequence;


#[derive(Debug, Clone)]
/// A sequence to contain the elements of a power series, where each term is a * t ^ n
/// ```
/// # use adic::sequence::Sequence;
/// let a = adic::sequence::factory::power(1, 5, 1);
/// assert_eq!(vec![1, 5, 25, 125, 625, 3125], a.truncation(6));
/// ```
pub struct PowerSequence<T>
where T: LocalOne + Clone {
    // { coefficient * term ^ (power * n) }
    coefficient: T,
    term: T,
    power: u32,
}

impl<T> PowerSequence<T>
where T: LocalOne + Clone {
    /// Constructor
    pub fn new(coefficient: T, term: T, power: u32) -> Self {
        Self {coefficient, term, power}
    }
}

impl<T> PowerSequence<T>
where T: LocalOne + Clone + UltraNormed {
    /// The index at which the valuation of the `Sequence` term is greater than or equal to `valuation`
    ///
    /// Returns `None` if `valuation` is `PosInf`, if `term` is adic-fractional (with a non-zero power), and if `term` is a unit or the power is zero.
    pub fn critical_valuation_index<V> (&self, valuation: V) -> Option<usize>
    where V: Into<Valuation<T::ValuationRing>> {
        use Valuation::{Finite, PosInf};
        //v = v_c + v_t * i * p
        let converted_power = T::ValuationRing::try_from_u32(self.power).expect("Convert u32 to ValuationRing");
        match (valuation.into(), self.coefficient.valuation(), self.term.valuation()) {
            // Looking for `PosInf` valuation
            (PosInf, _ , _) => None,
            // Coefficient is zero
            (Finite(_v), PosInf, _) => Some(0),
            // Term is adic-fractional so the valuation grows unbounded (as long as the power is non-zero)
            (Finite(_v), Finite(_v_coeff), Finite(v_term)) if ( v_term < T::ValuationRing::zero() && !converted_power.is_zero() ) => None,
            // Valuation of coefficient is >= v
            (Finite(v), Finite(v_coeff), _) if ( v_coeff >= v ) => Some(0),
            // Term is a unit or power is zero and v_coeff is < v so it will never grow larger
            (Finite(_v), Finite(_v_coeff), Finite(v_term)) if ( v_term.is_zero() || converted_power.is_zero() ) => None,
            // Term is zero so once n is one the valuation is greater than finite
            (Finite(_v), Finite(_v_coeff), PosInf) => Some(1),
            // Finds the index by calculating (v - v_coeff) / (v_term * power)
            (Finite(v), Finite(v_coeff), Finite(v_term)) => {
                let v_diff_coeff = (v - v_coeff).try_into_usize().expect("Convert ValuationRing to usize");
                let v_term_power = (v_term * converted_power).try_into_usize().expect("Convert ValuationRing to usize");
                Some(v_diff_coeff.div_ceil(v_term_power))
            },
        }
    }

    /// The valuation of the Sequence at `index`
    pub fn valuation_at(&self, index: usize) -> Valuation<T::ValuationRing> {
        //coefficient * term ^ (n * power)
        let converted_index = T::ValuationRing::try_from_usize(index).expect("Convert usize to ValuationRing").into();
        let converted_power = T::ValuationRing::try_from_u32(self.power).expect("Convert u32 to ValuationRing").into();
        self.coefficient.valuation() + self.term.valuation() * converted_index * converted_power
    }
}

impl<T> Sequence for PowerSequence<T> 
where T: LocalOne + Clone {
    type Term = T;
    fn terms(&self) -> Box<dyn Iterator<Item = Self::Term> + '_> {
        let a = (0..self.power).fold(self.term.local_one(), |acc, _| acc * self.term.clone());
        let mut prev_term = self.coefficient.clone();
        Box::new(once(prev_term.clone()).chain(repeat_with(move || {
            prev_term = prev_term.clone() * a.clone();
            prev_term.clone()
        })))
    }

    fn approx_num_terms(&self) -> Option<usize> {
        None
    }

}

#[cfg(test)]
mod test{
    use crate::{
        normed::Valuation,
        traits::PrimedFrom,
        EAdic, QAdic,
    };
    use super::PowerSequence;

    #[test]
    fn critical_valuation_index() {
        let a = PowerSequence::new(EAdic::primed_from(5, 1), EAdic::primed_from(5, 5), 1);
        assert_eq!(a.critical_valuation_index(3), Some(3));
        assert_eq!(a.critical_valuation_index(Valuation::PosInf), None);

        let a = PowerSequence::new(EAdic::primed_from(5, 0), EAdic::primed_from(5, 5), 1);
        assert_eq!(a.critical_valuation_index(0), Some(0));
        assert_eq!(a.critical_valuation_index(3), Some(0));

        let a = PowerSequence::new(EAdic::primed_from(5, 25), EAdic::primed_from(5, 5), 1);
        assert_eq!(a.critical_valuation_index(0), Some(0));
        assert_eq!(a.critical_valuation_index(1), Some(0));
        assert_eq!(a.critical_valuation_index(2), Some(0));
        assert_eq!(a.critical_valuation_index(3), Some(1));
        assert_eq!(a.critical_valuation_index(4), Some(2));
        assert_eq!(a.critical_valuation_index(5), Some(3));
        assert_eq!(a.critical_valuation_index(6), Some(4));

        let a = PowerSequence::new(EAdic::primed_from(5, 25), EAdic::primed_from(5, 0), 1);
        assert_eq!(a.critical_valuation_index(0), Some(0));
        assert_eq!(a.critical_valuation_index(1), Some(0));
        assert_eq!(a.critical_valuation_index(2), Some(0));
        assert_eq!(a.critical_valuation_index(3), Some(1));
        assert_eq!(a.critical_valuation_index(4), Some(1));
        assert_eq!(a.critical_valuation_index(5), Some(1));

        let a = PowerSequence::new(EAdic::primed_from(5, 25), EAdic::primed_from(5, 6), 1);
        assert_eq!(a.critical_valuation_index(3), None);

        let a = PowerSequence::new(EAdic::primed_from(5, 25), EAdic::primed_from(5, 5), 0);
        assert_eq!(a.critical_valuation_index(3), None);

        let a = PowerSequence::new(EAdic::primed_from(5, 25), EAdic::primed_from(5, 5), 2);
        assert_eq!(a.critical_valuation_index(0), Some(0));
        assert_eq!(a.critical_valuation_index(1), Some(0));
        assert_eq!(a.critical_valuation_index(2), Some(0));
        assert_eq!(a.critical_valuation_index(3), Some(1));
        assert_eq!(a.critical_valuation_index(4), Some(1));
        assert_eq!(a.critical_valuation_index(5), Some(2));
        assert_eq!(a.critical_valuation_index(6), Some(2));
        assert_eq!(a.critical_valuation_index(7), Some(3));
        assert_eq!(a.critical_valuation_index(8), Some(3));
        assert_eq!(a.critical_valuation_index(9), Some(4));
        assert_eq!(a.critical_valuation_index(10), Some(4));
        assert_eq!(a.critical_valuation_index(11), Some(5));

        let zero = QAdic::new(EAdic::primed_from(5, 0), 0);
        let one = QAdic::new(EAdic::primed_from(5, 1), 0);
        let five = QAdic::new(EAdic::primed_from(5, 1), 1);
        let one_fifth = QAdic::new(EAdic::primed_from(5, 1), -1);
        let one_twenty_fifth = QAdic::new(EAdic::primed_from(5, 1), -2);
        let a = PowerSequence::new(one, one_fifth.clone(), 1);
        assert_eq!(a.critical_valuation_index(0), None);
        assert_eq!(a.critical_valuation_index(1), None);
        assert_eq!(a.critical_valuation_index(2), None);

        let a = PowerSequence::new(zero, one_fifth.clone(), 1);
        assert_eq!(a.critical_valuation_index(0), Some(0));

        let a = PowerSequence::new(five.clone(), one_fifth, 0);
        assert_eq!(a.critical_valuation_index(0), Some(0));
        assert_eq!(a.critical_valuation_index(1), Some(0));
        assert_eq!(a.critical_valuation_index(2), None);

        let a = PowerSequence::new(one_twenty_fifth.clone(), five.clone(), 1);
        assert_eq!(a.critical_valuation_index(-3), Some(0));
        assert_eq!(a.critical_valuation_index(-2), Some(0));
        assert_eq!(a.critical_valuation_index(-1), Some(1));
        assert_eq!(a.critical_valuation_index(0), Some(2));
        assert_eq!(a.critical_valuation_index(1), Some(3));
        assert_eq!(a.critical_valuation_index(2), Some(4));

        let a = PowerSequence::new(one_twenty_fifth.clone(), five.clone(), 2);
        assert_eq!(a.critical_valuation_index(-3), Some(0));
        assert_eq!(a.critical_valuation_index(-2), Some(0));
        assert_eq!(a.critical_valuation_index(-1), Some(1));
        assert_eq!(a.critical_valuation_index(0), Some(1));
        assert_eq!(a.critical_valuation_index(1), Some(2));
        assert_eq!(a.critical_valuation_index(2), Some(2));
    }
}
