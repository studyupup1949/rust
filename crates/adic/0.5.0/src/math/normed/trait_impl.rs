use num::{BigInt, BigUint, Signed, One, Zero};
use super::Normed;


// Normed

macro_rules! impl_normed_uint {
    ( $Int:tt ) => {
        impl Normed for $Int {
            type Norm = Self;
            type Unit = Self;
            fn norm(&self) -> Self::Norm {
                *self
            }
            fn unit(&self) -> Option<Self::Unit> {
                if self.is_zero() {
                    None
                } else {
                    Some(1)
                }
            }
            fn into_unit(self) -> Option<Self::Unit> {
                self.unit()
            }
            fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self {
                if norm.is_zero() {
                    Self::zero()
                } else {
                    norm * u
                }
            }
            fn from_unit(u: Self::Unit) -> Self {
                u
            }
            fn is_unit(&self) -> bool {
                self.is_one()
            }
        }
    }
}

impl_normed_uint!(u8);
impl_normed_uint!(u16);
impl_normed_uint!(u32);
impl_normed_uint!(u64);
impl_normed_uint!(u128);
impl_normed_uint!(usize);


macro_rules! impl_normed_iint {
    ( $Int:ty, $Norm:ty ) => {
        impl Normed for $Int {
            type Norm = $Norm;
            type Unit = $Int;
            fn norm(&self) -> Self::Norm {
                self.unsigned_abs()
            }
            fn unit(&self) -> Option<Self::Unit> {
                if self.is_positive() {
                    Some(1)
                } else if self.is_negative() {
                    Some(-1)
                } else {
                    None
                }
            }
            fn into_unit(self) -> Option<Self::Unit> {
                self.unit()
            }
            fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self {
                if norm.is_zero() {
                    Self::zero()
                } else {
                    Self::try_from(norm).unwrap() * u
                }
            }
            fn from_unit(u: Self::Unit) -> Self {
                u
            }
            fn is_unit(&self) -> bool {
                [1, -1].contains(self)
            }
        }
    }
}

impl_normed_iint!(i8, u8);
impl_normed_iint!(i16, u16);
impl_normed_iint!(i32, u32);
impl_normed_iint!(i64, u64);
impl_normed_iint!(i128, u128);
impl_normed_iint!(isize, usize);


impl Normed for BigUint {
    type Norm = Self;
    type Unit = Self;
    fn norm(&self) -> Self::Norm {
        self.clone()
    }
    fn unit(&self) -> Option<Self::Unit> {
        if self.is_zero() {
            None
        } else {
            Some(Self::one())
        }
    }
    fn into_unit(self) -> Option<Self::Unit> {
        self.unit()
    }
    fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self {
        if norm.is_zero() {
            Self::zero()
        } else {
            norm * u
        }
    }
    fn from_unit(u: Self::Unit) -> Self {
        u
    }
    fn is_unit(&self) -> bool {
        self.is_one()
    }
}

impl Normed for BigInt {
    type Norm = BigUint;
    type Unit = BigInt;
    fn norm(&self) -> Self::Norm {
        self.magnitude().clone()
    }
    fn unit(&self) -> Option<Self::Unit> {
        if self.is_positive() {
            Some(Self::one())
        } else if self.is_negative() {
            Some(-Self::one())
        } else {
            None
        }
    }
    fn into_unit(self) -> Option<Self::Unit> {
        self.unit()
    }
    fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self {
        if norm.is_zero() {
            Self::zero()
        } else {
            Self::from(norm) * u
        }
    }
    fn from_unit(u: Self::Unit) -> Self {
        u
    }
    fn is_unit(&self) -> bool {
        [Self::one(), -Self::one()].contains(self)
    }
}



#[cfg(test)]
mod test {
    use super::Normed;

    #[test]
    fn normed() {

        assert!(1.is_unit());
        assert!(!0.is_unit());
        assert!(!2.is_unit());
        assert!(!3.is_unit());
        assert!((-1).is_unit());
        assert!(!(-2).is_unit());

        assert_eq!(Some(1), 2.unit());
        assert_eq!(None, 0.unit());
        assert_eq!(Some(-1), (-2).unit());

    }

}
