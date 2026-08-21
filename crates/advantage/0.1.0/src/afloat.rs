use super::*;
use num::traits::{Float, Num, NumCast, One, ToPrimitive, Zero};

/// Single precision `AFloat`
pub type ASingle = AFloat<f32>;
/// Double precision `AFloat`
pub type ADouble = AFloat<f64>;

/// Floating point variable type for tapeless forward-mode automatic differentiation
#[derive(Clone, Copy, Debug)]
pub struct AFloat<S: Float> {
    /// Zero-order value
    v: S,
    /// First order value
    dv: S,
    /// Context id and value id
    ctx: Option<(usize, usize)>,
}

impl<S: Float> AFloat<S> {
    /// Create a variable from its zero- and first-order value
    pub fn new(v: S, dv: S) -> Self {
        Self { v, dv, ctx: None }
    }

    /// Get the zero-order value
    pub fn value(&self) -> S {
        self.v
    }

    /// Borrow the zero-order value mutably
    pub fn value_mut(&mut self) -> &mut S {
        &mut self.v
    }

    /// Get the first-order value
    pub fn dvalue(&self) -> S {
        self.dv
    }

    /// Borrow the first-order value mutably
    pub fn dvalue_mut(&mut self) -> &mut S {
        &mut self.dv
    }

    /// Operation
    fn from_op(opcode: OpCode, mut arg1: Self, mut arg2: Option<Self>) -> Self {
        let v = zero_order_value(opcode, arg1.v, arg2.map(|x| x.v));
        let dv = first_order_value(
            opcode,
            arg1.v,
            arg2.map(|x| x.v),
            arg1.dv,
            arg2.map(|x| x.dv),
        );
        let mut this = Self::new(v, dv);

        let mut cid = None;
        // Get context id
        if let Some((cid1, _)) = arg1.context() {
            cid = Some(cid1);
        }
        if let Some(arg2) = arg2 {
            if let Some((cid2, _)) = arg2.context() {
                if let Some(cid) = cid {
                    assert_eq!(cid2, cid);
                } else {
                    cid = Some(cid2);
                }
            }
        }
        if let Some(cid) = cid {
            // Get context
            if let Some(mut ctx) = AContext::from_cid(cid) {
                // Add constants if necessary
                if arg1.context().is_none() {
                    let vid = ctx.record(OpCode::Const, arg1.value(), None, None);
                    arg1.ctx = Some((cid, vid));
                }
                if let Some(ref mut arg2) = &mut arg2 {
                    if arg2.context().is_none() {
                        let vid = ctx.record(OpCode::Const, arg2.value(), None, None);
                        arg2.ctx = Some((cid, vid));
                    }
                }
                // Get value ids
                let arg1_vid = Some(arg1.context().unwrap().1);
                let arg2_vid = arg2.map(|arg2| arg2.context().unwrap().1);
                // Record operation
                let vid = ctx.record(opcode, v, arg1_vid, arg2_vid);
                this.ctx = Some((cid, vid));
            }
        }
        this
    }

    /// Cast to different value type
    pub fn cast<T: Float>(x: AFloat<T>) -> Self {
        Self {
            v: S::from(x.v).unwrap(),
            dv: S::from(x.dv).unwrap(),
            ctx: x.ctx,
        }
    }

    pub(crate) fn set_context(&mut self, ctx_id: usize, val_id: usize) {
        self.ctx = Some((ctx_id, val_id));
    }

    pub(crate) fn context(&self) -> Option<(usize, usize)> {
        self.ctx
    }
}

impl<S: Float> std::cmp::PartialEq<AFloat<S>> for AFloat<S> {
    fn eq(&self, other: &Self) -> bool {
        self.v.eq(&other.v)
    }
}

impl<S: Float> std::cmp::PartialOrd<AFloat<S>> for AFloat<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.v.partial_cmp(&other.v)
    }
}

impl<S: Float> From<S> for AFloat<S> {
    fn from(scalar: S) -> Self {
        AFloat::new(scalar, S::zero())
    }
}

impl<S: Float> std::ops::Neg for AFloat<S> {
    type Output = AFloat<S>;
    fn neg(self) -> Self {
        (Self::zero() - Self::one()) * self
    }
}

macro_rules! binary_op {
    ($op:ident, $method:ident) => {
        impl<S: Float> std::ops::$op<AFloat<S>> for AFloat<S> {
            type Output = AFloat<S>;
            fn $method(self, rhs: Self) -> Self {
                Self::from_op(OpCode::$op, self, Some(rhs))
            }
        }

        impl<S: Float> std::ops::$op<f64> for AFloat<S> {
            type Output = AFloat<S>;
            fn $method(self, rhs: f64) -> Self {
                Self::from_op(OpCode::$op, self, Some(NumCast::from(rhs).unwrap()))
            }
        }

        impl<S: Float> std::ops::$op<AFloat<S>> for f64 {
            type Output = AFloat<S>;
            fn $method(self, rhs: AFloat<S>) -> AFloat<S> {
                AFloat::<S>::from_op(OpCode::$op, NumCast::from(self).unwrap(), Some(rhs))
            }
        }
    };
}

binary_op!(Add, add);
binary_op!(Sub, sub);
binary_op!(Mul, mul);
binary_op!(Div, div);

macro_rules! assign_op {
    ($op:ident, $method:ident, $optoken:tt) => {
        impl<S: Float> std::ops::$op<AFloat<S>> for AFloat<S> {
            fn $method(&mut self, rhs: AFloat<S>) {
                let result = *self $optoken rhs;
                *self = result;
            }
        }

        impl<S: Float> std::ops::$op<f64> for AFloat<S> {
            fn $method(&mut self, rhs: f64) {
                let result = *self $optoken rhs;
                *self = result;
            }
        }
    }
}

assign_op!(AddAssign, add_assign, +);
assign_op!(SubAssign, sub_assign, -);
assign_op!(MulAssign, mul_assign, *);
assign_op!(DivAssign, div_assign, /);

impl<S: Float> std::ops::Rem<AFloat<S>> for AFloat<S> {
    type Output = AFloat<S>;
    fn rem(self, _rhs: Self) -> Self {
        panic!("Operation '%' unsupported on AFloat");
    }
}

impl<S: Float> ToPrimitive for AFloat<S> {
    fn to_i64(&self) -> Option<i64> {
        self.v.to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.v.to_u64()
    }
}

impl<S: Float> NumCast for AFloat<S> {
    fn from<T>(n: T) -> Option<Self>
    where
        T: ToPrimitive,
    {
        S::from(n).map(|n| Self::new(n, S::zero()))
    }
}

impl<S: Float> Zero for AFloat<S> {
    fn zero() -> Self {
        Self::new(S::zero(), S::zero())
    }

    fn is_zero(&self) -> bool {
        self.v.is_zero()
    }
}

impl<S: Float> One for AFloat<S> {
    fn one() -> Self {
        Self::new(S::one(), S::zero())
    }

    fn is_one(&self) -> bool {
        self.v.is_one()
    }
}

impl<S: Float> Num for AFloat<S> {
    type FromStrRadixErr = S::FromStrRadixErr;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::new(S::from_str_radix(str, radix)?, S::zero()))
    }
}

macro_rules! float_constant {
    ($method:ident) => {
        fn $method() -> Self {
            Self::new(S::$method(), S::zero())
        }
    };
}

macro_rules! float_passthrough {
    ($type:ty, $method:ident) => {
        fn $method(self) -> $type {
            self.v.$method()
        }
    }
}

macro_rules! float_unsupported {
    ($type:ty, $method:ident $(, $arg_type:ty)*) => {
        fn $method(self, $(_: $arg_type)*) -> $type {
            panic!(concat!("Operation '", stringify!($method), "' unsupported on AFloat"));
        }
    }
}

macro_rules! float_elemental {
    ($method:ident, $opcode:ident) => {
        fn $method(self) -> Self {
            Self::from_op(OpCode::$opcode, self, None)
        }
    }
}

macro_rules! float_elemental2 {
    ($method:ident, $opcode:ident) => {
        fn $method(self, other: Self) -> Self {
            Self::from_op(OpCode::$opcode, self, Some(other))
        }
    }

}

impl<S: Float> Float for AFloat<S> {
    float_constant!(nan);
    float_constant!(infinity);
    float_constant!(neg_infinity);
    float_constant!(neg_zero);
    float_constant!(min_value);
    float_constant!(min_positive_value);
    float_constant!(max_value);

    float_passthrough!(bool, is_nan);
    float_passthrough!(bool, is_infinite);
    float_passthrough!(bool, is_finite);
    float_passthrough!(bool, is_normal);
    float_passthrough!(bool, is_sign_positive);
    float_passthrough!(bool, is_sign_negative);

    float_passthrough!(std::num::FpCategory, classify);

    float_unsupported!(Self, floor);
    float_unsupported!(Self, ceil);
    float_unsupported!(Self, round);
    float_unsupported!(Self, trunc);
    float_unsupported!(Self, fract);
    float_unsupported!(Self, signum);
    float_unsupported!(Self, exp_m1);
    float_unsupported!(Self, ln_1p);
    float_unsupported!(Self, sinh);
    float_unsupported!(Self, cosh);
    float_unsupported!(Self, tanh);
    float_unsupported!(Self, asinh);
    float_unsupported!(Self, acosh);
    float_unsupported!(Self, atanh);
    float_unsupported!(Self, atan2, Self);

    float_elemental!(abs, Abs);
    float_elemental!(exp, Exp);
    float_elemental!(ln, Ln);
    float_elemental!(sin, Sin);
    float_elemental!(cos, Cos);
    float_elemental!(tan, Tan);
    float_elemental!(asin, Asin);
    float_elemental!(acos, Acos);
    float_elemental!(atan, Atan);
    float_elemental2!(powf, Powf);

    fn mul_add(self, a: Self, b: Self) -> Self {
        (self * a) + b
    }

    fn recip(self) -> Self {
        Self::one() / self
    }

    fn powi(self, n: i32) -> Self {
        self.powf(<Self as NumCast>::from(n).unwrap())
    }

    fn sqrt(self) -> Self {
        self.powf(<Self as NumCast>::from(0.5).unwrap())
    }

    fn exp2(self) -> Self {
        <Self as NumCast>::from(2.0).unwrap().powf(self)
    }

    fn log(self, base: Self) -> Self {
        self.ln() / base.ln()
    }

    fn log2(self) -> Self {
        self.log(<Self as NumCast>::from(2.0).unwrap())
    }

    fn log10(self) -> Self {
        self.log(<Self as NumCast>::from(10.0).unwrap())
    }

    fn max(self, other: Self) -> Self {
        <Self as NumCast>::from(0.5).unwrap() * (self + other + (self - other).abs())
    }

    fn min(self, other: Self) -> Self {
        <Self as NumCast>::from(0.5).unwrap() * (self + other - (self - other).abs())
    }

    fn abs_sub(self, other: Self) -> Self {
        (self - other).abs()
    }

    fn cbrt(self) -> Self {
        self.powf(Self::one() / <Self as NumCast>::from(3.0).unwrap())
    }

    fn hypot(self, other: Self) -> Self {
        (self.powi(2) + other.powi(2)).sqrt()
    }

    fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
    }

    fn integer_decode(self) -> (u64, i16, i8) {
        self.v.integer_decode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const EPS: f64 = 1e-5;

    #[allow(clippy::let_and_return)]
    fn test_function<S: Float>(x: S) -> S {
        let v1 = x + x - x * x / x;
        let v2 = (v1 + S::from(2.0).unwrap() - S::from(2.0).unwrap()) * S::from(2.0).unwrap()
            / S::from(2.0).unwrap();
        let v3 = -(S::from(2.0).unwrap() - (S::from(2.0).unwrap() + v2));
        let v4 = S::from(2.0).unwrap() / (S::from(2.0).unwrap() * v3);
        let v5 = S::one() / v4;
        v5
    }

    #[test]
    fn afloat_consistency() {
        let x = AFloat::<f64>::new(2.0, 1.0);
        let y = test_function(x);
        assert!((y.value() - x.value()).abs() < std::f64::EPSILON);
        assert!((y.dvalue() - 1.0).abs() < std::f64::EPSILON);
    }

    #[test]
    fn afloat_nonlinear_functions() {
        macro_rules! test_case {
            ($func:ident, $dy:expr) => {
                let x = AFloat::<f64>::new(0.5, 1.0);
                let y = x.$func();
                println!("{}, {}, {}", stringify!($func), y.dvalue(), $dy(x.value()));
                assert!(y.dvalue() - $dy(x.value()) < EPS)
            };
        }
        test_case!(sin, |x: f64| x.cos());
        test_case!(cos, |x: f64| -x.sin());
        test_case!(tan, |x: f64| (1.0 / x.cos()).powi(2));
        test_case!(exp, |x: f64| x.exp());
        test_case!(ln, |x: f64| 1.0 / x);
        test_case!(sqrt, |x: f64| 0.5 / x.sqrt());
        test_case!(asin, |x: f64| 1.0 / (1.0 - x.powi(2)).sqrt());
        test_case!(acos, |x: f64| -1.0 / (1.0 - x.powi(2)).sqrt());
        test_case!(atan, |x: f64| 1.0 / (1.0 + x.powi(2)));
    }
}
