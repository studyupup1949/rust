#[macro_export]
/// Create a [`UAdic`](crate::UAdic) number more concisely
///
/// ```
/// # use adic::uadic;
/// assert_eq!("431._5", uadic!(5, [1, 3, 4]).to_string());
/// ```
macro_rules! uadic {
    ( $p:expr, [$( $fixed_digits:expr ),* $(,)?] ) => {
        $crate::UAdic::new($p, vec![$($fixed_digits,)*])
    };
}


#[macro_export]
/// Create a [`RAdic`](crate::RAdic) number more concisely
///
/// ```
/// # use adic::radic;
/// assert_eq!("...321321654._7", radic!(7, [4, 5, 6], [1, 2, 3]).to_string());
/// ```
macro_rules! radic {
    ( $p:expr, [$( $fixed_digits:expr ),*], [$( $repeating_digits:expr ),* $(,)?] ) => {
        $crate::RAdic::new($p, vec![$($fixed_digits,)*], vec![$($repeating_digits,)*])
    };
}

#[macro_export]
/// Create an approximate [`ZAdic`](crate::ZAdic) number more concisely
///
/// ```
/// # use adic::zadic_approx;
/// assert_eq!("---004310._5", zadic_approx!(5, 6, [0, 1, 3, 4]).to_string());
/// ```
macro_rules! zadic_approx {
    ( $p:expr, $precision:expr, [$( $known_digits:expr ),* $(,)?] ) => {
        $crate::ZAdic::new_approx($p, $precision, vec![$($known_digits,)*])
    };
}

#[macro_export]
/// Create a positive exact [`ZAdic`](crate::ZAdic) number more concisely
///
/// ```
/// # use adic::zadic_exact;
/// assert_eq!("4310._5", zadic_exact!(5, [0, 1, 3, 4]).to_string());
/// ```
macro_rules! zadic_exact {
    ( $p:expr, [$( $known_digits:expr ),* $(,)?]) => {
        $crate::ZAdic::new_exact($p, vec![$($known_digits,)*])
    };
}

#[macro_export]
/// Create a negative exact [`ZAdic`](crate::ZAdic) number more concisely
///
/// ```
/// # use adic::zadic_exact_neg;
/// assert_eq!("...44310._5", zadic_exact_neg!(5, [0, 1, 3]).to_string());
/// ```
macro_rules! zadic_exact_neg {
    ( $p:expr, [$( $known_digits:expr ),* $(,)?]) => {
        $crate::ZAdic::new_exact_neg($p, vec![$($known_digits,)*])
    };
}

#[macro_export]
/// Create a [`ZAdicVariety`](crate::ZAdicVariety) more concisely
///
/// ```
/// # use adic::zadic_variety;
/// assert_eq!(
///     "variety(---3210._5, ---0401._5)",
///     zadic_variety!(5, 4, [[0, 1, 2, 3], [1, 0, 4]]).to_string()
/// );
/// ```
/// `zadic_variety!(p, precision=5, \[ \[a, b, c\], \[d, e\]\]) = { ???00cba._p, ???000de._p }`
macro_rules! zadic_variety {
    ( $p:expr, $precision:expr, [$( [$( $known_digits:expr ),* $(,)?] ),* $(,)?] ) => {
        $crate::ZAdicVariety::new($p, vec![
            $($crate::ZAdic::new_approx($p, $precision, vec![$($known_digits,)*]), )*
        ])
    };
}


#[cfg(test)]
mod test {

    use num::Rational32;
    use crate::{RAdic, UAdic, ZAdic, ZAdicVariety};

    #[test]
    pub fn test_macros() {
        let u = uadic!(5, [1, 2, 3]);
        assert_eq!(UAdic::new(5, vec![1, 2, 3]), u);
        assert_eq!(3*25 + 2*5 + 1, u.integer_value());
        let r = radic!(5, [1, 2, 3], [1, 0]);
        assert_eq!(RAdic::new(5, vec![1, 2, 3], vec![1, 0]), r);
        assert_eq!(
            Rational32::from_integer(3*25 + 2*5 + 1) + Rational32::new(-125, 24),
            r.rational_value()
        );
        let za = zadic_approx!(5, 4, [1, 2, 3]);
        assert_eq!(ZAdic::new_approx(5, 4, vec![1, 2, 3]), za);
        let ze = zadic_exact!(5, [1, 2, 3]);
        assert_eq!(ZAdic::new_exact(5, vec![1, 2, 3]), ze);
        let zv = zadic_variety!(5, 3, [[1, 2, 3], [4, 5, 6]]);
        assert_eq!(ZAdicVariety::new(5, vec![
            zadic_approx!(5, 3, [1, 2, 3]),
            zadic_approx!(5, 3, [4, 5, 6])
        ]), zv);
    }

}
