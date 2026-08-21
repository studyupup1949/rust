use num::{One, Zero};
use crate::traits::AdicPrimitive;
use super::maybe_primed::{MaybePrimed, RingBacking};


impl<A, N> Zero for MaybePrimed<A, N>
where A: AdicPrimitive, N: RingBacking<A> {
    fn zero() -> Self {
        Self::unprimed(N::zero())
    }
    fn is_zero(&self) -> bool {
        match self {
            Self::Primed(a) => a.is_local_zero(),
            Self::Unprimed(n) => n.is_zero(),
        }
    }
}

impl<A, N> One for MaybePrimed<A, N>
where A: AdicPrimitive, N: RingBacking<A> {
    fn one() -> Self {
        Self::unprimed(N::one())
    }
}


impl<A, N> std::ops::Add for MaybePrimed<A, N>
where A: AdicPrimitive, N: RingBacking<A> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Primed(a), Self::Primed(b)) => Self::Primed(a + b),
            (Self::Primed(a), Self::Unprimed(n)) => {
                let p = a.p();
                Self::Primed(a + n.primed_into(p))
            },
            (Self::Unprimed(m), Self::Primed(b)) => {
                let p = b.p();
                Self::Primed(m.primed_into(p) + b)
            },
            (Self::Unprimed(m), Self::Unprimed(n)) => Self::Unprimed(m + n),
        }
    }
}

impl<A, N> std::ops::Neg for MaybePrimed<A, N>
where A: AdicPrimitive + std::ops::Neg<Output=A>,
N: RingBacking<A> + std::ops::Neg<Output=N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Primed(a) => Self::Primed(-a),
            Self::Unprimed(n) => Self::Unprimed(-n),
        }
    }
}

impl<A, N> std::ops::Sub for MaybePrimed<A, N>
where A: AdicPrimitive + std::ops::Sub<Output=A>,
N: RingBacking<A> + std::ops::Sub<Output=N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Primed(a), Self::Primed(b)) => Self::Primed(a - b),
            (Self::Primed(a), Self::Unprimed(n)) => {
                let p = a.p();
                Self::Primed(a - n.primed_into(p))
            },
            (Self::Unprimed(m), Self::Primed(b)) => {
                let p = b.p();
                Self::Primed(m.primed_into(p) - b)
            },
            (Self::Unprimed(m), Self::Unprimed(n)) => Self::Unprimed(m - n),
        }
    }
}

impl<A, N> std::ops::Mul for MaybePrimed<A, N>
where A: AdicPrimitive, N: RingBacking<A> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Primed(a), Self::Primed(b)) => Self::Primed(a * b),
            (Self::Primed(a), Self::Unprimed(n)) => {
                let p = a.p();
                Self::Primed(a * n.primed_into(p))
            },
            (Self::Unprimed(m), Self::Primed(b)) => {
                let p = b.p();
                Self::Primed(m.primed_into(p) * b)
            },
            (Self::Unprimed(m), Self::Unprimed(n)) => Self::Unprimed(m * n),
        }
    }
}

impl<A, N, RHS> num::traits::Pow<RHS> for MaybePrimed<A, N>
where A: AdicPrimitive + num::traits::Pow<RHS, Output=A>,
N: RingBacking<A> + num::traits::Pow<RHS, Output=N> {
    type Output = Self;
    fn pow(self, rhs: RHS) -> Self::Output {
        match self {
            Self::Primed(a) => Self::Primed(a.pow(rhs)),
            Self::Unprimed(n) => Self::Unprimed(n.pow(rhs)),
        }
    }
}
