use std::iter::{once, repeat_n};
use crate::{
    local_num::{LocalOne, LocalZero},
    num_adic::{IAdic, RAdic, UAdic},
    traits::CanTruncate,
};


#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub (crate) enum IntegerVariant {
    /// `Unsigned` holds a `UAdic`
    Unsigned(UAdic),
    /// `Signed` holds an `IAdic`
    Signed(IAdic),
    /// `Rational` holds a `RAdic`
    Rational(RAdic),
}


impl IntegerVariant {

    pub (crate) fn truncate_and_then<F, O>(&mut self, c: usize, f: F) -> O
    where F: Fn(&mut UAdic) -> O {
        match self {
            IntegerVariant::Unsigned(u) if u.finite_num_digits() > c => {
                let mut u = u.truncation(c);
                let o = f(&mut u);
                *self = IntegerVariant::Unsigned(u);
                o
            },
            IntegerVariant::Unsigned(u) => {
                f(u)
            },
            IntegerVariant::Signed(i) => {
                let mut u = i.truncation(c);
                let o = f(&mut u);
                *self = IntegerVariant::Unsigned(u);
                o
            },
            IntegerVariant::Rational(r) => {
                let mut u = r.truncation(c);
                let o = f(&mut u);
                *self = IntegerVariant::Unsigned(u);
                o
            },
        }
    }

    pub (crate) fn truncate(&mut self, c: usize) {
        self.truncate_and_then(c, |_| { });
    }

    pub (crate) fn truncate_and_push(&mut self, c: usize, digit: u32) {
        self.truncate_and_then(c, |u| {
            u.extend_digits(&repeat_n(0, c - u.finite_num_digits()).chain(once(digit)).collect::<Vec<_>>());
        });
    }

    pub (crate) fn is_local_zero(&self) -> bool {
        match self {
            Self::Unsigned(a) => a.is_local_zero(),
            Self::Signed(a) => a.is_local_zero(),
            Self::Rational(a) => a.is_local_zero(),
        }
    }

    pub (crate) fn is_local_one(&self) -> bool {
        match self {
            Self::Unsigned(a) => a.is_local_one(),
            Self::Signed(a) => a.is_local_one(),
            Self::Rational(a) => a.is_local_one(),
        }
    }

}


impl From<UAdic> for IntegerVariant {
    fn from(value: UAdic) -> Self {
        Self::Unsigned(value)
    }
}

impl From<IAdic> for IntegerVariant {
    fn from(value: IAdic) -> Self {
        Self::Signed(value)
    }
}

impl From<RAdic> for IntegerVariant {
    fn from(value: RAdic) -> Self {
        Self::Rational(value)
    }
}
