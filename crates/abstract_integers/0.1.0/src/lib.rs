//! This crate defines specification-friendly natural integers with an upper bound. Operations on
//! these integers can be defined as modular (modulo the upper bound) or regular (with a panic
//! on underflow or overflow).
//!
//! # Defining a new integer type
//!
//! Here is the macro used to defined the `SizeNat` type of this crate:
//!
//! ```ignore
//! define_abstract_integer_checked!(SizeNat, 8, BigUint::from(std::usize::MAX));
//! ```
//!
//! `SizeNat` is the name of the newly-created type. `8` is the length in bytes of the byte array
//! that will hold values of this type. The third argument is the upper bound of this integer type,
//! given as a `num::BigUint`.
//!
//! The resulting integer type is copyable, and supports addition, substraction, multiplication,
//! integer division, remainder, comparison and equality. The `from_literal` method allows you to
//! convert integer literals into your new type.
//!
//! # Example
//!
//! ```
//! # use num::BigUint;
//! # use abstract_integers::*;
//! let x1 = SizeNat::from_literal(687165654266415);
//! let x2 = SizeNat::from_literal(4298832000156);
//! let x3 = x1 + x2;
//! assert_eq!(SizeNat::from_literal(691464486266571), x3);
//! let x4 = SizeNat::from_literal(8151084996540);
//! let x5 = x3 - x4;
//! assert_eq!(SizeNat::from_literal(683313401270031), x5.into());
//! let x6 = x5 / SizeNat::from_literal(1541654268);
//! assert_eq!(SizeNat::from_literal(443233), x6.into());
//! ```
//!


extern crate num;
#[allow(unused_imports)]
use num::{BigUint, CheckedSub, Zero};
use std::ops::*;

macro_rules! define_abstract_integer_checked_operations {
    ($name:ident, $bytes:literal) => {
        /// **Warning**: panics on overflow.
        impl Add for $name {
            type Output = $name;
            fn add(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c = a + b;
                if c > $name::max() {
                    panic!("bounded addition overflow for type {}", stringify!($name));
                }
                c.into()
            }
        }

        /// **Warning**: panics on underflow.
        impl Sub for $name {
            type Output = $name;
            fn sub(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c = a.checked_sub(&b).unwrap_or_else(|| {
                    panic!(
                        "bounded substraction underflow for type {}",
                        stringify!($name)
                    )
                });
                c.into()
            }
        }

        /// **Warning**: panics on overflow.
        impl Mul for $name {
            type Output = $name;
            fn mul(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c = a * b;
                if c > $name::max() {
                    panic!("bounded addition overflow for type {}", stringify!($name));
                }
                c.into()
            }
        }

        /// **Warning**: panics on division by 0.
        impl Div for $name {
            type Output = $name;
            fn div(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                if b == BigUint::zero() {
                    panic!("dividing by zero in type {}", stringify!($name));
                }
                let c = a / b;
                c.into()
            }
        }

        /// **Warning**: panics on division by 0.
        impl Rem for $name {
            type Output = $name;
            fn rem(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                if b == BigUint::zero() {
                    panic!("dividing by zero in type {}", stringify!($name));
                }
                let c = a % b;
                c.into()
            }
        }
    };
}

macro_rules! define_abstract_integer_modular_operations {
    ($name:ident, $bytes:literal) => {
        /// **Warning**: wraps on overflow.
        impl Add for $name {
            type Output = $name;
            fn add(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c: BigUint = a + b;
                let d: BigUint = c % $name::max();
                d.into()
            }
        }

        /// **Warning**: wraps on underflow.
        impl Sub for $name {
            type Output = $name;
            fn sub(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c: BigUint = if b > a { $name::max() - b + a } else { b - a };
                c.into()
            }
        }

        /// **Warning**: wraps on overflow.
        impl Mul for $name {
            type Output = $name;
            fn mul(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                let c: BigUint = a * b;
                let d: BigUint = c % $name::max();
                d.into()
            }
        }

        /// **Warning**: panics on division by 0.
        impl Div for $name {
            type Output = $name;
            fn div(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                if b == BigUint::zero() {
                    panic!("dividing by zero in type {}", stringify!($name));
                }
                let c: BigUint = a / b;
                c.into()
            }
        }

        /// **Warning**: panics on division by 0.
        impl Rem for $name {
            type Output = $name;
            fn rem(self, rhs: $name) -> $name {
                let a: BigUint = self.into();
                let b: BigUint = rhs.into();
                if b == BigUint::zero() {
                    panic!("dividing by zero in type {}", stringify!($name));
                }
                let c: BigUint = a % b;
                c.into()
            }
        }
    };
}

macro_rules! define_abstract_integer_common_operations {
    ($name:ident, $bytes:literal) => {
        impl $name {
            /// Returns 2 to the power of the argument
            #[allow(dead_code)]
            pub fn pow2(x: usize) -> $name {
                BigUint::from(1u32).shl(x).into()
            }
        }

        impl PartialEq for $name {
            fn eq(&self, rhs: &$name) -> bool {
                let a : BigUint = (*self).into();
                let b : BigUint = (*rhs).into();
                a == b
            }
        }

        impl Eq for $name {}

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &$name) -> Option<std::cmp::Ordering> {
                let a : BigUint = (*self).into();
                let b : BigUint = (*other).into();
                a.partial_cmp(&b)
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &$name) -> std::cmp::Ordering {
                self.partial_cmp(other).unwrap()
            }
        }

    };
}

macro_rules! define_abstract_integer_struct {
    ($name:ident, $bytes:literal, $max:expr) => {
        #[derive(Clone, Copy)]
        pub struct $name([u8; $bytes]);

        impl From<BigUint> for $name {
            fn from(x: BigUint) -> $name {
                let repr = x.to_bytes_be();
                if repr.len() > $bytes {
                    panic!("BigUint too big for type {}", stringify!($name))
                }
                let mut out = [0u8; $bytes];
                let upper = out.len();
                let lower = upper - repr.len();
                out[lower..upper].copy_from_slice(&repr);
                $name(out)
            }
        }

        impl Into<BigUint> for $name {
            fn into(self) -> BigUint {
                BigUint::from_bytes_be(&self.0)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let uint: BigUint = (*self).into();
                write!(f, "{}", uint)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let uint: BigUint = (*self).into();
                write!(f, "{}", uint)
            }
        }

        impl $name {
            fn max() -> BigUint {
                $max
            }

            #[allow(dead_code)]
            pub fn from_literal(x: u128) -> Self {
                let big_x = BigUint::from(x);
                if big_x > $name::max().into() {
                    panic!("literal too big for type {}", stringify!($name));
                }
                big_x.into()
            }
        }
    };
}

/// Defines a bounded natural integer with modular arithmetic operations
#[macro_export]
macro_rules! define_abstract_integer_modular {
    ($name:ident, $bytes:literal, $max:expr) => {
        define_abstract_integer_struct!($name, $bytes, $max);
        define_abstract_integer_common_operations!($name, $bytes);
        define_abstract_integer_modular_operations!($name, $bytes);
    };
}

/// Defines a bounded natural integer with regular arithmetic operations, checked for overflow
/// and underflow.
#[macro_export]
macro_rules! define_abstract_integer_checked {
    ($name:ident, $bytes:literal, $max:expr) => {
        define_abstract_integer_struct!($name, $bytes, $max);
        define_abstract_integer_common_operations!($name, $bytes);
        define_abstract_integer_checked_operations!($name, $bytes);
    };
}

/// Natural integer bounded by std::usize::MAX
define_abstract_integer_checked!(SizeNat, 8, BigUint::from(std::usize::MAX));

mod tests;
