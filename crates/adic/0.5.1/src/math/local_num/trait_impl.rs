use num::{Integer, Num, One, Zero};
use super::{LocalInteger, LocalNum, LocalOne, LocalZero};


impl <T> LocalZero for T
where T: Zero {
    fn local_zero(&self) -> Self {
        Self::zero()
    }
    fn is_local_zero(&self) -> bool {
        self.is_zero()
    }
}

// Note that we have to add `PartialEq` as requirement, otherwise One::is_one is not accessible.
// See https://github.com/rust-num/num-traits/issues/47.
impl <T> LocalOne for T
where T: One + PartialEq {
    fn local_one(&self) -> Self {
        Self::one()
    }
    fn is_local_one(&self) -> bool {
        self.is_one()
    }
}

impl<T> LocalNum for T
where T: Num {
    type FromStrRadixErr = <T as Num>::FromStrRadixErr;
    fn from_str_radix(
        s: &str,
        radix: u32,
    ) -> Result<Self, Self::FromStrRadixErr> {
        <Self as Num>::from_str_radix(s, radix)
    }
}

impl<T> LocalInteger for T
where T: Integer {
    fn gcd(&self, other: &Self) -> Self {
        <Self as Integer>::gcd(self, other)
    }
    fn lcm(&self, other: &Self) -> Self {
        <Self as Integer>::lcm(self, other)
    }
    fn is_multiple_of(&self, other: &Self) -> bool {
        <Self as Integer>::is_multiple_of(self, other)
    }
}
