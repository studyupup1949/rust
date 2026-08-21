#![allow(unused_macros)]

// All doctests are annotated with "compile_fail" to make sure they don't compile outside the crate

/// Create a [`UAdic`](crate::UAdic) number more concisely
///
/// ```compile_fail
/// assert_eq!("231._5", uadic!(5, [1, 3, 2]).to_string());
/// ```
macro_rules! uadic {
    ( $p:expr, [$( $fixed_digits:expr ),* $(,)?] ) => {
        $crate::UAdic::new($p, vec![$($fixed_digits,)*])
    };
}


/// Create a natural [`EAdic`](crate::EAdic) number more concisely
///
/// ```compile_fail
/// assert_eq!("231._5", eadic!(5, [1, 3, 2]).to_string());
/// ```
macro_rules! eadic {
    ( $p:expr, [$( $fixed_digits:expr ),* $(,)?] ) => {
        $crate::EAdic::new($p, vec![$($fixed_digits,)*])
    };
}

/// Create a negative integer [`EAdic`](crate::EAdic) number more concisely
///
/// ```compile_fail
/// assert_eq!("(4)231._5", eadic_neg!(5, [1, 3, 2]).to_string());
/// ```
macro_rules! eadic_neg {
    ( $p:expr, [$( $fixed_digits:expr ),* $(,)?] ) => {
        $crate::EAdic::new_neg($p, vec![$($fixed_digits,)*])
    };
}

/// Create a rational/repeating digit [`EAdic`](crate::EAdic) number more concisely
///
/// ```compile_fail
/// assert_eq!("(321)654._7", eadic_rep!(7, [4, 5, 6, 1, 2], [3, 1, 2]).to_string());
/// ```
macro_rules! eadic_rep {
    ( $p:expr, [$( $fixed_digits:expr ),*], [$( $repeating_digits:expr ),* $(,)?] ) => {
        $crate::EAdic::new_repeating($p, vec![$($fixed_digits,)*], vec![$($repeating_digits,)*])
    };
}


/// Create an approximate [`ZAdic`](crate::ZAdic) number more concisely
///
/// ```compile_fail
/// assert_eq!("...004310._5", zadic_approx!(5, 6, [0, 1, 3, 4]).to_string());
/// ```
macro_rules! zadic_approx {
    ( $p:expr, $precision:expr, [$( $known_digits:expr ),* $(,)?] ) => {
        $crate::ZAdic::new_approx($p, $precision, vec![$($known_digits,)*])
    };
}


/// Create a [`Variety`](crate::Variety) more concisely
///
/// ```compile_fail
/// assert_eq!(
///     "variety(...3210._5, ...0401._5)",
///     zadic_variety!(5, 4, [[0, 1, 2, 3], [1, 0, 4]]).to_string()
/// );
/// ```
/// `zadic_variety!(p, precision=5, [ [a, b, c], [d, e] ]) = { ...00cba._p, ...000de._p }`
macro_rules! zadic_variety {
    ( $p:expr, $precision:expr, [$( [$( $known_digits:expr ),* $(,)?] ),* $(,)?] ) => {
        $crate::Variety::<$crate::ZAdic>::new(vec![
            $($crate::ZAdic::new_approx($p, $precision, vec![$($known_digits,)*]), )*
        ])
    };
}


/// Create a [`QAdic`](crate::QAdic) more concisely
///
/// ```compile_fail
/// assert_eq!(
///     "43.21_5",
///     qadic!(uadic!(5, [1, 2, 3, 4]), -2).to_string()
/// );
/// ```
macro_rules! qadic {
    ( $a:expr, $v:expr ) => {
        $crate::QAdic::new($a, $crate::normed::Valuation::Finite($v))
    };
}


/// Create a [`PowAdic`](crate::num_adic::PowAdic) more concisely
///
/// ```compile_fail
/// assert_eq!(
///     "4321._9",
///     apow!(uadic!(3, [1, 0, 2, 0, 0, 1, 1, 1]), 2).to_string()
/// );
/// ```
macro_rules! apow {
    ( $a:expr, $pp:expr ) => {
        $crate::num_adic::PowAdic::new($a, $pp)
    };
}



// OPS MACROS

macro_rules! impl_add_assign_from_add {
    ( $T:ty ) => {
        impl std::ops::AddAssign for $T {
            fn add_assign(&mut self, rhs: Self) {
                *self = self.clone() + rhs;
            }
        }
    }
}

macro_rules! impl_sub_from_neg {
    ( $T:ty) => {
        impl ops::Sub for $T {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                self + (-rhs)
            }
        }
    }
}

macro_rules! impl_sub_assign_from_sub {
    ( $T:ty ) => {
        impl std::ops::SubAssign for $T {
            fn sub_assign(&mut self, rhs: Self) {
                *self = self.clone() - rhs;
            }
        }
    }
}

macro_rules! impl_mul_assign_from_mul {
    ( $T:ty ) => {
        impl std::ops::MulAssign for $T {
            fn mul_assign(&mut self, rhs: Self) {
                *self = self.clone() * rhs;
            }
        }
    }
}

macro_rules! impl_pow_by_squaring {
    ( $T:ty ) => {

        impl Pow<u32> for $T {
            type Output = $T;
            fn pow(self, mut power: u32) -> Self::Output {

                // Exponentiation by squaring
                let mut out = <$T>::one(self.p());
                if power == 0 {
                    return out;
                }

                let mut mult = self;
                while power > 1 {
                    if power.is_odd() {
                        out = out * mult.clone();
                        power = power - 1;
                    }
                    mult = mult.clone() * mult;
                    power = power / 2;
                }
                out * mult

            }
        }

    }
}



#[cfg(test)]
mod test {

    use num::Rational32;
    use crate::{EAdic, QAdic, UAdic, Variety, ZAdic};

    #[test]
    fn macros() {
        let u = uadic!(5, [1, 2, 3]);
        assert_eq!(UAdic::new(5, vec![1, 2, 3]), u);
        assert_eq!(Ok(3*25 + 2*5 + 1), u.u32_value());
        let r = eadic_rep!(5, [1, 2, 3], [1, 0]);
        assert_eq!(EAdic::new_repeating(5, vec![1, 2, 3], vec![1, 0]), r);
        assert_eq!(
            Ok(Rational32::from_integer(3*25 + 2*5 + 1) + Rational32::new(-125, 24)),
            r.rational_value()
        );
        let za = zadic_approx!(5, 4, [1, 2, 3]);
        assert_eq!(ZAdic::new_approx(5, 4, vec![1, 2, 3]), za);
        let zv = zadic_variety!(5, 3, [[1, 2, 3], [4, 3, 2]]);
        assert_eq!(Variety::new(vec![
            zadic_approx!(5, 3, [1, 2, 3]),
            zadic_approx!(5, 3, [4, 3, 2])
        ]), zv);
        let q = qadic!(uadic!(5, [1, 2, 3, 4]), -2);
        assert_eq!(QAdic::new(UAdic::new(5, vec![1, 2, 3, 4]), crate::normed::Valuation::Finite(-2)), q);
    }

}
