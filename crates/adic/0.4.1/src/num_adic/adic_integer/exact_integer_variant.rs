use std::iter::{once, repeat_n};
use crate::{
    AdicInteger, IAdic, RAdic, UAdic,
};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Variant of the three types of exact `AdicIntegers`:
///  unsigned `UAdic`, signed `IAdic`, rational `RAdic`
pub (crate) enum ExactIntegerVariant {
    /// `Unsigned` holds a `UAdic`
    Unsigned(UAdic),
    /// `Signed` holds an `IAdic`
    Signed(IAdic),
    /// `Rational` holds a `RAdic`
    Rational(RAdic),
}


impl ExactIntegerVariant {

    pub (crate) fn truncation(&self, c: usize) -> UAdic {
        match self {
            Self::Unsigned(u) if u.finite_num_digits() > c => u.truncation(c),
            Self::Unsigned(u) => u.clone(),
            Self::Signed(i) => i.truncation(c),
            Self::Rational(r) => r.truncation(c),
        }
    }

    pub (crate) fn truncate_and_then<F, O>(&mut self, c: usize, f: F) -> O
    where F: Fn(&mut UAdic) -> O {
        match self {
            Self::Unsigned(u) if u.finite_num_digits() > c => {
                let mut u = u.truncation(c);
                let o = f(&mut u);
                *self = ExactIntegerVariant::Unsigned(u);
                o
            },
            Self::Unsigned(u) => {
                f(u)
            },
            Self::Signed(i) => {
                let mut u = i.truncation(c);
                let o = f(&mut u);
                *self = ExactIntegerVariant::Unsigned(u);
                o
            },
            Self::Rational(r) => {
                let mut u = r.truncation(c);
                let o = f(&mut u);
                *self = ExactIntegerVariant::Unsigned(u);
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

    pub (crate) fn truncate_and_pop(&mut self, c: usize) -> Option<u32> {
        self.truncate_and_then(c, |u| {
            if c > u.finite_num_digits() {
                Some(0)
            } else {
                u.pop_digit()
            }
        })
    }

}
