use std::rc::Rc;
use num::{One, Zero};
use std::ops::{Add, Div, Mul, Sub};
use crate::{
    function::series::DelayedPowerSeries,
    local_num::{LocalOne, LocalZero},
    mapping::{IndexedMapping, Mapping},
    sequence,
    PowerSeries, Sequence,
};


/// Geometric series `x^n`
pub fn geometric<'t, T>() -> impl IndexedMapping<T, Output=T> + 't
where T: Clone + LocalZero + LocalOne + 't {
    let s = |t: T| {
        Rc::new(sequence::factory::constant(t.local_one()))
        as Rc<dyn Sequence<Term=T>>
    };
    DelayedPowerSeries::new(s)
}

/// Exponential series, `x^n/n!`
pub fn exp<'t, T>() -> impl IndexedMapping<T, Output=T> + 't
where T: Clone + PartialEq + LocalZero + LocalOne + Add<Output = T> + Mul<Output = T> + Div<Output = T> + 't {
    let s = |t: T| {
        Rc::new(sequence::factory::exponential_progression(t.local_one()))
        as Rc<dyn Sequence<Term=T>>
    };
    DelayedPowerSeries::new(s)
}

/// ln(x) from taylor series `(-1)^(n+1) * (x - 1)^n / n`
///
/// Converges only for `|x - 1| < 1`
pub fn ln<'t, T>() -> impl IndexedMapping<T, Output=T> + 't
where T: Clone + PartialEq + Zero + One + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + From<i32> + 't {

    let s = |n: usize| {
        if n.is_zero() {
            T::zero()
        } else if n % 2 == 1 {
            T::one() / T::from(i32::try_from(n).unwrap())
        } else {
            (T::zero()-T::one()) / T::from(i32::try_from(n).unwrap())
        }
    };

    let series = PowerSeries::new(s);
    let inner = |x| x - T::one();
    series.compose(inner)

}

/// ln(z) from atanh Taylor series expansion
///
/// (2n + 1)-th term = `2 * 1/(2n+1) * [(x - 1) / (x + 1)]^(2n+1)`
///         = `2 * [(x - 1) / (x + 1)] * 1/(2n+1) * [(x - 1)^2 / (x + 1)^2]^n`
pub fn ln_from_atanh<'t, T>() -> impl IndexedMapping<T, Output=T> + 't
where T: Clone + PartialEq + Zero + One + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + From<i32> + 't {

    let s = |n: usize| {
        if n.is_multiple_of(2) {
            T::zero()
        } else {
            T::from(2) / T::from(i32::try_from(n).unwrap())
        }
    };

    let series = PowerSeries::new(s);
    let inner = |x: T| (x.clone() - T::one()) / (x.clone() + T::one());
    series.compose(inner)

}
