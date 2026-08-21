//! A crate that implements a half-precision floating-point type `f16` and its associated methods.

use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use std::fmt;

extern "C" {
    fn hs_floatToHalf(f: f32) -> u16; 
    fn hs_halfToFloat(c: u16) -> f32; 
}

/// Type `f16`, which is essentially a wrapper around u16.
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub struct f16 {
    bit_repr: u16,
}

const EXP_BW: u16 = 5;
const SIG_BW: u16 = 10;
const SIG_MASK: u16 = 1023;
const EXP_MASK: u16 = 31;
const POS_INF_BR: u16 = 31744;
const NEG_INF_BR: u16 = 64512;
const POS_ZERO_BR: u16 = 0;
const NEG_ZERO_BR: u16 = 32768;

impl f16 {
    // bit conversions
    /// Build a `f16` from its bit-representation.
    #[inline]
    pub fn from_bits(x: u16) -> Self {
        Self {
            bit_repr: x,
        }
    }

    /// Obtain a `f16`'s bit-representation.
    #[inline]
    pub fn to_bits(self) -> u16 {
        self.bit_repr
    }

    fn get_exp_bits(self) -> u16 {
        (self.to_bits() >> SIG_BW) & EXP_MASK
    }

    fn get_sig_bits(self) -> u16 {
        self.to_bits() & SIG_MASK
    }

    fn get_sign_bit(self) -> u16 {
        self.to_bits() >> (SIG_BW + EXP_BW)
    }

    // predicates
    /// The `is_finite` method for `f16`
    #[inline]
    pub fn is_finite(self) -> bool {
        self.get_exp_bits() < EXP_MASK
    }

    /// The `is_infinite` method for `f16`
    #[inline]
    pub fn is_infinite(self) -> bool {
        let b = self.to_bits();
        b == POS_INF_BR || b == NEG_INF_BR
    }

    /// The `is_nan` method for `f16`
    #[inline]
    pub fn is_nan(self) -> bool {
        self != self
    }

    /// The `is_normal` method for `f16`
    #[inline]
    pub fn is_normal(self) -> bool {
        let exp = self.get_exp_bits();
        exp > 0 && exp < EXP_MASK
    }

    /// The `is_sign_positive` method for `f16`
    #[inline]
    pub fn is_sign_positive(self) -> bool {
        self.get_sign_bit() == 0
    }

    /// The `is_sign_negative` method for `f16`
    #[inline]
    pub fn is_sign_negative(self) -> bool {
        !self.is_sign_positive()
    }

    fn is_subnormal(self) -> bool {
        self.get_exp_bits() == 0 && !self.is_zero()
    }

    fn is_zero(self) -> bool {
        let b = self.to_bits();
        b == POS_ZERO_BR || b == NEG_ZERO_BR
    }
}

// conversions from/to f32
impl From<f16> for f32 {
    fn from(x: f16) -> Self {
        unsafe {
            hs_halfToFloat(f16::to_bits(x))
        }
    }
}

impl From<f32> for f16 {
    fn from(x: f32) -> Self {
        unsafe {
            f16::from_bits(hs_floatToHalf(x))
        }
    }
}


macro_rules! bin_op {
    ($op_name:ident, $ret_type:ty) => {
        fn $op_name(&self, other: &Self) -> $ret_type {
            let lhs = f32::from(*self);
            let rhs = &f32::from(*other);
            lhs.$op_name(rhs)
        }
    }
}

macro_rules! bin_arith {
    ($op_trait:ident, $op_name:ident) => {
        impl $op_trait for f16 {
            type Output = f16;

            fn $op_name(self, other: Self) -> Self {
                let lhs = f32::from(self);
                let rhs = f32::from(other);
                Self::from(lhs.$op_name(rhs))
            }
        }
    }
}

// partial equality
impl PartialEq for f16 {
    bin_op!(eq, bool);
    bin_op!(ne, bool);
}

// partial order
impl PartialOrd for f16 {
    bin_op!(partial_cmp, Option<Ordering>);
}

// arithmetic
bin_arith!(Add, add);
bin_arith!(Sub, sub);
bin_arith!(Mul, mul);
bin_arith!(Div, div);

// printer
impl fmt::Display for f16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f32::from(*self).fmt(f)
    }
}


#[cfg(test)]
mod tests {
    use crate::f16;
    #[test]
    fn size_check() {
        use std::mem;
        assert_eq!(mem::size_of::<f16>(), 2);
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn identity_check() {
        let x = 1;
        let xs = f16 {
            bit_repr: x,
        };
        assert_eq!(x, xs.bit_repr);
    }

    #[test]
    fn bits_and_half() {
        let x = 1;
        assert_eq!(x, f16::from_bits(x).to_bits());
    }

    #[test]
    fn float_to_half() {
        let x0: f32 = 0.0;
        let x1: f32 = 1.0;
        let x2: f32 = 2.0;
        assert_eq!(0, f16::to_bits(f16::from(x0)));
        assert_eq!(15360, f16::from(x1).to_bits());
        assert_eq!(16384, f16::from(x2).to_bits());
    }

    #[test]
    fn half_to_float() {
        let x0: f16 = f16::from_bits(0);
        let x1: f16 = f16::from_bits(15360);
        let xmin: f16 = f16::from_bits(1);
        assert_eq!(0.0, f32::from(x0));
        assert_eq!(1.0, f32::from(x1));
        assert_eq!(5.9604645e-8, f32::from(xmin));
    }

    #[test]
    fn partial_eq_check() {
        let x0: f16 = f16::from_bits(0);
        let x1: f16 = f16::from_bits(1);
        assert!(x0 != x1);
    }

    #[test]
    fn partial_ord_check() {
        let x0: f16 = f16::from_bits(0);
        let x1: f16 = f16::from_bits(1);
        assert!(x0 < x1);
    }

    #[test]
    fn arith_ops() {
        let x0: f16 = f16::from_bits(0);
        let x1: f16 = f16::from_bits(1);
        assert_eq!(x1, x0+x1);
        assert_eq!(x0, x0*x1);
    }

    #[test]
    fn predicates() {
        let nan: f16 = f16::from(0.0/0.0);
        let pos_inf: f16 = f16::from(1.0/0.0);
        let neg_inf: f16 = f16::from(-1.0/0.0);
        let xmin: f16 = f16::from_bits(1);
        assert!(nan.is_nan());
        assert!(pos_inf.is_infinite());
        assert!(neg_inf.is_infinite());
        assert!(pos_inf.is_sign_positive());
        assert!(neg_inf.is_sign_negative());
        assert!(!xmin.is_normal());
        assert!(xmin.is_subnormal());
    }

}
