use num::{One, Zero};
use crate::{
    divisible::Prime,
    error::{AdicError, AdicResult},
    traits::{AdicPrimitive, PrimedInto},
};


/// A ring (implementing `Zero`, `One`, `Add`, `Mul`) that can be primed with `PrimedInto<A>`
pub (crate) trait RingBacking<A>: Clone + Zero + One
    + std::ops::Add<Self, Output=Self>
    + std::ops::Mul<Self, Output=Self>
    + PrimedInto<A> { }

impl<A, N> RingBacking<A> for N where N: Clone + Zero + One + PrimedInto<A> { }


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Prime or Unprimed enum
pub (crate) enum MaybePrimed<A, N>
where A: AdicPrimitive, N: RingBacking<A> {
    /// Primed, `A` impling `AdicPrimitive`
    Primed(A),
    /// Unprimed, can be primed into `A` with `PrimedFrom`
    Unprimed(N),
}


impl<A, N> MaybePrimed<A, N>
where A: AdicPrimitive, N: RingBacking<A> {

    /// Create `Primed`
    pub fn primed(a: A) -> Self {
        Self::Primed(a)
    }

    /// Create `Unprimed`
    pub fn unprimed(n: N) -> Self {
        Self::Unprimed(n)
    }

    /// Convert to primed with `p`
    pub fn prime_with<P>(self, p: P) -> AdicResult<A>
    where P: Into<Prime> {
        let p = p.into();
        match self {
            Self::Primed(a) if a.p() == p => Ok(a),
            Self::Primed(_) => Err(AdicError::MixedCharacteristic),
            Self::Unprimed(n) => Ok(n.primed_into(p)),
        }
    }

    /// Returns the prime for this `MaybePrimed` or `None` if it is `Unprimed`
    pub fn p(&self) -> Option<Prime> {
        match self {
            Self::Primed(a) => Some(a.p()),
            Self::Unprimed(_) => None,
        }
    }

    /// Convert to primed or error
    pub fn into_primed(self) -> AdicResult<A> {
        match self {
            Self::Primed(a) => Ok(a),
            Self::Unprimed(_) => Err(AdicError::NoPrimeSet),
        }
    }

    /// Convert between `MaybePrimed`
    pub fn convert<B, M>(self) -> MaybePrimed<B, M>
    where A: Into<B>, N: Into<M>, B: AdicPrimitive, M: RingBacking<B> {
        match self {
            Self::Primed(a) => MaybePrimed::Primed(a.into()),
            Self::Unprimed(n) => MaybePrimed::Unprimed(n.into()),
        }
    }

}


#[allow(dead_code)]
impl<'a, A, N> MaybePrimed<A, N>
where A: AdicPrimitive + 'a, N: RingBacking<A> + 'a {

    /// See if all `MaybePrimed` match prime exactly
    pub (crate) fn primes_match<P>(
        maybes: impl IntoIterator<Item=&'a Self>, p: P
    ) -> bool
    where P: Into<Prime> {
        let p = p.into();
        maybes.into_iter().all(|m| match m {
            Self::Primed(a) => a.p() == p,
            Self::Unprimed(_) => false,
        })
    }

    /// See if all `MaybePrimed` match prime or have no prime
    pub (crate) fn primes_match_or_empty<P>(
        maybes: impl IntoIterator<Item=&'a Self>, p: P
    ) -> bool
    where P: Into<Prime> {
        let p = p.into();
        maybes.into_iter().all(|m| match m {
            Self::Primed(a) => a.p() == p,
            Self::Unprimed(_) => true,
        })
    }

    /// - `Some(Prime)` for all exactly matching `MaybePrimed`
    /// - `None` for empty iterator
    /// - `[AdicError::MixedCharacteristic]` for mixed primes
    pub (crate) fn prime_for_all_matching(
        maybes: impl IntoIterator<Item=&'a Self>
    ) -> AdicResult<Option<Prime>> {
        let mut p = None;
        let all_match = maybes.into_iter().all(|maybe|
            match (maybe, p) {
                (Self::Primed(a), Some(p0)) => p0 == a.p(),
                (Self::Primed(a), None) => {
                    p = Some(a.p());
                    true
                },
                (Self::Unprimed(_), _) => false,
            }
        );
        match (all_match, p) {
            (true, p @ Some(_)) => Ok(p),
            (true, None) => Ok(None),
            (false, _) => Err(AdicError::MixedCharacteristic),
        }
    }

    /// - `Some(Prime)` for all exactly matching `MaybePrimed`
    /// - `None` for empty iterator or no `MaybePrimed::Primed`
    /// - `[AdicError::MixedCharacteristic]` for mixed primes
    pub (crate) fn prime_for_all_matching_or_empty(
        maybes: impl IntoIterator<Item=&'a Self>
    ) -> AdicResult<Option<Prime>> {
        let mut p = None;
        let all_match = maybes.into_iter().all(|maybe|
            match (maybe, p) {
                (Self::Primed(a), Some(p0)) => p0 == a.p(),
                (Self::Primed(a), None) => {
                    p = Some(a.p());
                    true
                },
                (Self::Unprimed(_), _) => true,
            }
        );
        match (all_match, p) {
            (true, p @ Some(_)) => Ok(p),
            (true, None) => Ok(None),
            (false, _) => Err(AdicError::MixedCharacteristic),
        }
    }

    /// See if all match a shared prime exactly, returning true for empty iterator
    pub (crate) fn all_match(
        maybes: impl IntoIterator<Item=&'a Self>
    ) -> bool {
        Self::prime_for_all_matching(maybes).is_ok()
    }

    /// See if all match a shared prime or are empty
    pub (crate) fn all_match_or_empty(
        maybes: impl IntoIterator<Item=&'a Self>
    ) -> bool {
        Self::prime_for_all_matching_or_empty(maybes).is_ok()
    }

}
