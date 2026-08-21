//! Adic integer algebraic variety

use std::cmp::Ordering;
use itertools::{EitherOrBoth, Itertools};
use crate::{
    error::{validate_matching_p, AdicError, AdicResult},
    divisible::Prime,
    traits::{AdicInteger, AdicPrimitive, CanApproximate, HasApproximateDigits, PrimedFrom, TryPrimedFrom},
    QAdic,
};
use super::Variety;


impl<A> Variety<A>
where A: AdicPrimitive {

    /// Prime for this adic variety, or error if the variety is empty
    pub fn p(&self) -> AdicResult<Prime> {
        validate_matching_p(self.elements.iter().map(A::p));
        self.elements.first().map(A::p).ok_or(AdicError::NoPrimeSet)
    }

    /// Create a `Variety` from a `Vec` of roots that can be primed into `A`
    pub fn primed_from_roots<P, B, IB>(p: P, roots: IB) -> Self
    where P: Into<Prime>, A: PrimedFrom<B>, IB: IntoIterator<Item = B> {
        let p = p.into();
        let roots = roots.into_iter().map(|root| A::primed_from(p, root)).collect();
        Variety::new(roots)
    }

    /// Create a `Variety` from a `Vec` of roots that can attempt to be primed into `A`
    pub fn try_primed_from_roots<P, B, IB>(p: P, roots: IB) -> AdicResult<Self>
    where P: Into<Prime>, A: TryPrimedFrom<B>, IB: IntoIterator<Item = B>, AdicError: From<A::Error> {
        let p = p.into();
        let roots = roots
            .into_iter()
            .map(|root| A::try_primed_from(p, root))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Variety::new(roots))
    }

    /// Approximate all members of this variety to certainty `n`
    ///
    /// ```
    /// # use adic::{UAdic, Variety, ZAdic};
    /// let av = Variety::new(vec![UAdic::new(5, vec![]), UAdic::new(5, vec![1]), UAdic::new(5, vec![1, 2, 3, 4])]);
    /// let zv = Variety::new(vec![ZAdic::new_approx(5, 2, vec![]), ZAdic::new_approx(5, 2, vec![1]), ZAdic::new_approx(5, 2, vec![1, 2])]);
    /// assert_eq!(zv, av.approximation(2));
    /// ```
    pub fn approximation(&self, n: A::DigitIndex) -> Variety<A::Approximation>
    where A: CanApproximate, A::Approximation: AdicPrimitive {
        Variety::new(self.roots().map(|r| r.approximation(n)).collect())
    }

    /// Approximate all members of this variety to certainty `n`
    ///
    /// ```
    /// # use adic::{UAdic, Variety, ZAdic};
    /// let av = Variety::new(vec![UAdic::new(5, vec![]), UAdic::new(5, vec![1]), UAdic::new(5, vec![1, 2, 3, 4])]);
    /// let zv = Variety::new(vec![ZAdic::new_approx(5, 2, vec![]), ZAdic::new_approx(5, 2, vec![1]), ZAdic::new_approx(5, 2, vec![1, 2])]);
    /// assert_eq!(zv, av.into_approximation(2));
    /// ```
    pub fn into_approximation(self, n: A::DigitIndex) -> Variety<A::Approximation>
    where A: CanApproximate, A::Approximation: AdicPrimitive {
        Variety::new(self.into_roots().map(|r| r.into_approximation(n)).collect())
    }

    /// Sort (in-place) this adic `Variety`, using digits from least to most significance
    pub fn adic_sort(&mut self)
    where A: HasApproximateDigits {
        validate_matching_p(self.elements.iter().map(A::p));
        let sorted_roots = sort_roots(self.elements.clone()).collect::<Vec<_>>();
        self.elements = sorted_roots;
    }

    #[must_use]
    /// Sort this adic `Variety`, using digits from least to most significance
    pub fn adic_sorted(self) -> Self
    where A: HasApproximateDigits {
        validate_matching_p(self.elements.iter().map(A::p));
        let sorted_roots = sort_roots(self.elements).collect::<Vec<_>>();
        Self {
            elements: sorted_roots,
        }
    }

}


impl<A> Variety<A>
where A: AdicPrimitive + AdicInteger {
    /// Convert adic integer variety into `QAdic` variety
    pub fn into_fractional(self) -> Variety<QAdic<A>> {
         Variety::new(self.into_roots().map(|a| QAdic::new(a, 0)).collect())
    }
}

impl<A> Variety<QAdic<A>>
where A: AdicPrimitive + AdicInteger {
    /// Convert `QAdic` variety into adic integer variety, throwing an error if any root is fractional
    pub fn try_into_integer(self) -> AdicResult<Variety<A>> {
        let zadic_roots = self.into_roots().map(QAdic::try_into_integer).collect::<AdicResult<Vec<_>>>()?;
        Ok(Variety::new(zadic_roots))
    }
}


impl<A, B> PrimedFrom<Variety<B>> for Variety<A>
where A: AdicPrimitive + PrimedFrom<B> {
    fn primed_from<P>(p: P, n: Variety<B>) -> Self
    where P: Into<Prime> {
        let p = p.into();
        let roots = n.into_roots().map(|root| A::primed_from(p, root)).collect();
        Variety::new(roots)
    }
}



fn sort_roots<A, I: IntoIterator<Item = A>>(roots: I) -> impl Iterator<Item = A>
where A: AdicPrimitive + HasApproximateDigits {
    roots.into_iter().sorted_by(|z1, z2| {
        match z1.digits().zip_longest(z2.digits()).find(
            |dd| dd.as_ref().both().is_none_or(|(l, r)| l != r)
        ) {
            None => {
                if z1.is_certain() && !z2.is_certain() {
                    Ordering::Less
                } else if !z1.is_certain() && z2.is_certain() {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            },
            Some(EitherOrBoth::Left(_)) => Ordering::Greater,
            Some(EitherOrBoth::Right(_)) => Ordering::Less,
            Some(EitherOrBoth::Both(d1, d2)) => d1.cmp(&d2),
        }
    })
}
