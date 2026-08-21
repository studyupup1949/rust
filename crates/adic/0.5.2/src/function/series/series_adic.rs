#![allow(dead_code)]

use crate::{
    divisible::Prime,
    error::{AdicError, AdicResult},
    sequence::Sequence,
    traits::{AdicPrimitive, PrimedInto, PrimedFrom},
};
use super::{KernelSeries, PowerSeries};


impl<'a, A> KernelSeries<'a, A>
where A: AdicPrimitive {

    /// Create an adic polynomial by priming the given unprimed `coefficients` with prime `p`
    pub fn new_with_prime<P, S>(p: P, coefficients: S) -> Self
    where P: Into<Prime>, S: Sequence + 'a, S::Term: PrimedInto<A> {
        let p = p.into();
        let primed_coefficients = coefficients.term_map(move |n| n.primed_into(p));
        Self::new(primed_coefficients)
    }

    /// Prime for this adic polynomial
    pub fn p(&self) -> AdicResult<Prime> {
        self.term_sequence().term_local_prime().ok_or(AdicError::NoPrimeSet)
    }

}

impl<'a, A, N> PrimedFrom<KernelSeries<'a, N>> for KernelSeries<'a, A>
where A: AdicPrimitive, N: PrimedInto<A> {
    fn primed_from<P>(p: P, n: KernelSeries<'a, N>) -> Self
    where P: Into<Prime> {
        let p = p.into();
        Self::new_with_prime(p, n.term_sequence())
    }
}


impl<'a, A> PowerSeries<'a, A>
where A: AdicPrimitive {

    /// Create an adic polynomial by priming the given unprimed `coefficients` with prime `p`
    pub fn new_with_prime<P, S>(p: P, coefficients: S) -> Self
    where P: Into<Prime>, S: Sequence + 'a, S::Term: PrimedInto<A> {
        let p = p.into();
        let primed_coefficients = coefficients.term_map(move |n| n.primed_into(p));
        Self::new(primed_coefficients)
    }

    /// Prime for this adic polynomial
    pub fn p(&self) -> AdicResult<Prime> {
        self.coefficients().term_local_prime().ok_or(AdicError::NoPrimeSet)
    }

}

impl<'a, A, N> PrimedFrom<PowerSeries<'a, N>> for PowerSeries<'a, A>
where A: AdicPrimitive, N: PrimedInto<A> {
    fn primed_from<P>(p: P, n: PowerSeries<'a, N>) -> Self
    where P: Into<Prime> {
        let p = p.into();
        Self::new_with_prime(p, n.coefficients())
    }
}



#[cfg(test)]
mod tests {
    use crate::{
        divisible::Prime,
        mapping::Mapping,
        traits::{AdicPrimitive, PrimedFrom, PrimedInto},
        EAdic,
    };
    use super::{KernelSeries, PowerSeries};

    #[test]
    fn new() {
        let _ = KernelSeries::<EAdic>::new(vec![EAdic::one(5)]);
        let _ = PowerSeries::<EAdic>::new_with_prime(5, vec![1]);
        let _ = KernelSeries::<EAdic>::primed_from(5, KernelSeries::new(vec![1]));
        let _: PowerSeries<EAdic> = PowerSeries::new(vec![1]).primed_into(5);
    }

    #[test]
    #[should_panic(expected="MixedCharacteristic")]
    fn mismatched_p() {
        let series = PowerSeries::new(vec![EAdic::zero(3), EAdic::one(3)]);
        assert_eq!(Ok(Prime::from(3)), series.p());
        let _ = series.eval(EAdic::zero(5));
    }

}
