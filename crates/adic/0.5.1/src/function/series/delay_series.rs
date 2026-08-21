use std::{
    ops::{Add, Mul, Neg, Sub},
    rc::Rc,
};

use crate::{
    error::AdicResult,
    local_num::{LocalOne, LocalZero},
    mapping::{Differentiable, IndexedMapping, Mapping},
    sequence::{factory as sequence_factory, Sequence},
};
use super::PowerSeries;


#[derive(Clone)]
/// A [`PowerSeries`], but where the sequence is not evaluated until the series is evaluated
pub struct DelayedPowerSeries<'t, T>
where T: 't {
    delayed_sequence: Rc<dyn Fn(T) -> Rc<dyn Sequence<Term=T> + 't> + 't>
}

impl<'t, T> DelayedPowerSeries<'t, T>
where T: 't {

    /// Constructs a new `DelayedPowerSeries` with the given coefficients
    pub fn new<F>(coefficient_eval: F) -> Self
    where F: Fn(T) -> Rc<dyn Sequence<Term=T> + 't> + 't {
        Self {
            delayed_sequence: Rc::new(coefficient_eval),
        }
    }

}


impl<T> Mapping<T> for DelayedPowerSeries<'_, T>
where T: Clone + LocalZero + LocalOne + Mul<Output=T> {
    type Output = T;
    fn eval(&self, x: T) -> AdicResult<T> {
        let power_series = PowerSeries::new((self.delayed_sequence)(x.clone()));
        power_series.eval(x)
    }
}

impl<T> IndexedMapping<T> for DelayedPowerSeries<'_, T>
where T: Clone + LocalZero + LocalOne + Mul<Output=T> {
    fn eval_finite(&self, x: T, num_terms: usize) -> AdicResult<T> {
        let power_series = PowerSeries::new((self.delayed_sequence)(x.clone()));
        power_series.eval_finite(x, num_terms)
    }
}

impl<T> Add for DelayedPowerSeries<'_, T>
where T: Clone + Add<Output=T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        DelayedPowerSeries::new(
            move |t: T| Rc::new((self.delayed_sequence)(t.clone()).term_add((rhs.delayed_sequence)(t)))
        )
    }
}

impl<T> Neg for DelayedPowerSeries<'_, T>
where T: Neg<Output=T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        DelayedPowerSeries::new(
            move |t: T| Rc::new((self.delayed_sequence)(t).term_map(T::neg))
        )
    }
}

impl<T> Sub for DelayedPowerSeries<'_, T>
where T: Clone + Add<Output=T> + Neg<Output=T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl<T> Mul for DelayedPowerSeries<'_, T>
where T: Clone + Add<Output=T> + Mul<Output=T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        DelayedPowerSeries::new(
            move |t: T| Rc::new((self.delayed_sequence)(t.clone()).foil_mul((rhs.delayed_sequence)(t)))
        )
    }
}

impl<T> Differentiable for  DelayedPowerSeries<'_, T>
where T: Clone + LocalZero + LocalOne + Mul<Output=T> {
    type Output = Self;
    fn into_derivative(self) -> Self::Output {
        let new_coefficient_eval = move |t| {
            let sequence = (self.delayed_sequence)(t);
            let first = sequence.terms().next().clone();
            let deriv_sequence: Rc<dyn Sequence<Term=T>> = if let Some(first) = first {
                let one = first.local_one();
                let l = sequence_factory::linear(one);
                Rc::new(l.term_mul(sequence, false))
            } else {
                Rc::new(vec![])
            };
            deriv_sequence
        };
        Self::new(new_coefficient_eval)
    }
}
