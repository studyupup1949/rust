#[macro_export]
/// Create a [`UAdic`](crate::UAdic) number more concisely
///
/// ```
/// # use adic::uadic;
/// assert_eq!("231._5", uadic!(5, [1, 3, 2]).to_string());
/// ```
macro_rules! uadic {
    ( $p:expr, [$( $fixed_digits:expr ),* $(,)?] ) => {
        $crate::UAdic::new($p, vec![$($fixed_digits,)*])
    };
}


#[macro_export]
/// Create a positive [`IAdic`](crate::IAdic) number more concisely
///
/// ```
/// # use adic::iadic_pos;
/// assert_eq!("231._5", iadic_pos!(5, [1, 3, 2]).to_string());
/// ```
macro_rules! iadic_pos {
    ( $p:expr, [$( $fixed_digits:expr ),* $(,)?] ) => {
        $crate::IAdic::new_pos($p, vec![$($fixed_digits,)*])
    };
}


#[macro_export]
/// Create a negative [`IAdic`](crate::IAdic) number more concisely
///
/// ```
/// # use adic::iadic_neg;
/// assert_eq!("(4)231._5", iadic_neg!(5, [1, 3, 2]).to_string());
/// ```
macro_rules! iadic_neg {
    ( $p:expr, [$( $fixed_digits:expr ),* $(,)?] ) => {
        $crate::IAdic::new_neg($p, vec![$($fixed_digits,)*])
    };
}


#[macro_export]
/// Create a [`RAdic`](crate::RAdic) number more concisely
///
/// ```
/// # use adic::radic;
/// assert_eq!("(321)654._7", radic!(7, [4, 5, 6, 1, 2], [3, 1, 2]).to_string());
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
/// assert_eq!("...004310._5", zadic_approx!(5, 6, [0, 1, 3, 4]).to_string());
/// ```
macro_rules! zadic_approx {
    ( $p:expr, $precision:expr, [$( $known_digits:expr ),* $(,)?] ) => {
        $crate::ZAdic::new_approx($p, $precision, vec![$($known_digits,)*])
    };
}

#[macro_export]
/// Create an exact [`ZAdic`](crate::ZAdic) number more concisely
///  (you may find `ZAdic::from` more ergonomic)
///
/// ```
/// # use adic::{iadic_neg, radic, uadic, zadic_exact, ZAdic};
/// assert_eq!("4310._5", zadic_exact!(uadic!(5, [0, 1, 3, 4])).to_string());
/// assert_eq!("(4)310._5", zadic_exact!(iadic_neg!(5, [0, 1, 3])).to_string());
/// assert_eq!("(14)310._5", zadic_exact!(radic!(5, [0, 1, 3], [4, 1])).to_string());
/// assert_eq!("(14)310._5", ZAdic::from(radic!(5, [0, 1, 3], [4, 1])).to_string());
/// ```
macro_rules! zadic_exact {
    ( $a:expr ) => {
        $crate::ZAdic::from($a)
    };
}

#[macro_export]
#[deprecated]
/// Create a positive exact [`ZAdic`](crate::ZAdic) number more concisely
///
/// ```
/// # use adic::zadic_exact_pos;
/// assert_eq!("4310._5", zadic_exact_pos!(5, [0, 1, 3, 4]).to_string());
/// ```
macro_rules! zadic_exact_pos {
    ( $p:expr, [$( $known_digits:expr ),* $(,)?]) => {
        $crate::ZAdic::from($crate::uadic!($p, [$($known_digits,)*]))
    };
}

#[macro_export]
#[deprecated]
/// Create a negative exact [`ZAdic`](crate::ZAdic) number more concisely
///
/// ```
/// # use adic::zadic_exact_neg;
/// assert_eq!("(4)310._5", zadic_exact_neg!(5, [0, 1, 3]).to_string());
/// ```
macro_rules! zadic_exact_neg {
    ( $p:expr, [$( $known_digits:expr ),* $(,)?]) => {
        $crate::ZAdic::from($crate::iadic_neg!($p, [$($known_digits,)*]))
    };
}

#[macro_export]
/// Create a [`AdicVariety`](crate::AdicVariety) more concisely
///
/// ```
/// # use adic::zadic_variety;
/// assert_eq!(
///     "variety(...3210._5, ...0401._5)",
///     zadic_variety!(5, 4, [[0, 1, 2, 3], [1, 0, 4]]).to_string()
/// );
/// ```
/// `zadic_variety!(p, precision=5, [ [a, b, c], [d, e] ]) = { ...00cba._p, ...000de._p }`
macro_rules! zadic_variety {
    ( $p:expr, $precision:expr, [$( [$( $known_digits:expr ),* $(,)?] ),* $(,)?] ) => {
        $crate::ZAdicVariety::new($p, vec![
            $($crate::ZAdic::new_approx($p, $precision, vec![$($known_digits,)*]), )*
        ])
    };
}

#[macro_export]
/// Create a [`QAdic`](crate::QAdic) more concisely
///
/// ```
/// # use adic::{qadic, uadic};
/// assert_eq!(
///     "43.21_5",
///     qadic!(uadic!(5, [1, 2, 3, 4]), -2).to_string()
/// );
/// ```
macro_rules! qadic {
    ( $a:expr, $v:expr ) => {
        $crate::QAdic::new($a, $crate::AdicValuation::Finite($v))
    };
}

#[macro_export]
/// Create a [`AdicPower`](crate::AdicPower) more concisely
///
/// ```
/// # use adic::{apow, uadic};
/// assert_eq!(
///     "4321._9",
///     apow!(uadic!(3, [1, 0, 2, 0, 0, 1, 1, 1]), 2).to_string()
/// );
/// ```
macro_rules! apow {
    ( $a:expr, $pp:expr ) => {
        $crate::AdicPower::new($a, $pp)
    };
}

#[macro_export]
/// Create an [`AdicPolynomal`](crate::AdicPolynomial) with [`Iadic`](crate::IAdic) coefficients more concisely
///
/// ```
/// # use adic::iadic_poly;
/// assert_eq!(
///     "1._7x^2 + 0._7x^1 + (6)5._7x^0",
///     iadic_poly!(7, [-2, 0, 1]).to_string()
/// );
/// ```
macro_rules! iadic_poly {
    ( $p:expr, [$( $coefficients:expr ),* $(,)?] ) => {
        $crate::AdicPolynomial::<$crate::IAdic>::new($p, vec![
            $(<$crate::IAdic as $crate::SignedAdicNumber>::from_i32($p, $coefficients), )*
        ])
    };
}

#[macro_export]
/// Create an [`AdicPolynomal`](crate::AdicPolynomial) with [`ZAdic`](crate::ZAdic) coefficients more concisely
///
/// ```
/// # use adic::zadic_poly;
/// assert_eq!(
///     "1._7x^2 + 0._7x^1 + (6)5._7x^0",
///     zadic_poly!(7, [-2, 0, 1]).to_string()
/// );
/// ```
macro_rules! zadic_poly {
    ( $p:expr, [$( $coefficients:expr ),* $(,)?] ) => {
        $crate::AdicPolynomial::<$crate::ZAdic>::new($p, vec![
            $(<$crate::ZAdic as $crate::SignedAdicNumber>::from_i32($p, $coefficients), )*
        ])
    };
}

#[macro_export]
/// Create an [`AdicPolynomial`](crate::AdicPolynomial) with [`AdicInteger`](crate::AdicInteger) coefficients more concisely
///
/// ```
/// # use adic::{adic_poly, iadic_pos, iadic_neg};
/// assert_eq!(
///     "1._7x^2 + 0._7x^1 + (6)5._7x^0",
///     adic_poly!(7, [iadic_neg!(7, [5]), iadic_pos!(7, []), iadic_pos!(7, [1]),]).to_string()
/// );
/// ```
macro_rules! adic_poly {
    ( $p:expr, [$( $coefficients:expr ),* $(,)?] ) => {
        $crate::AdicPolynomial::new($p, vec![
            $( $coefficients, )*
        ])
    };
}


#[cfg(test)]
mod test {

    use num::Rational32;
    use crate::{AdicNumber, AdicPolynomial, IAdic, QAdic, RAdic, UAdic, ZAdic, AdicVariety};

    #[test]
    fn macros() {
        let u = uadic!(5, [1, 2, 3]);
        assert_eq!(UAdic::new(5, vec![1, 2, 3]), u);
        assert_eq!(3*25 + 2*5 + 1, u.u32_value());
        let r = radic!(5, [1, 2, 3], [1, 0]);
        assert_eq!(RAdic::new(5, vec![1, 2, 3], vec![1, 0]), r);
        assert_eq!(
            Rational32::from_integer(3*25 + 2*5 + 1) + Rational32::new(-125, 24),
            r.rational_value()
        );
        let za = zadic_approx!(5, 4, [1, 2, 3]);
        assert_eq!(ZAdic::new_approx(5, 4, vec![1, 2, 3]), za);
        let ze = zadic_exact!(uadic!(5, [1, 2, 3]));
        assert_eq!(ZAdic::from(uadic!(5, [1, 2, 3])), ze);
        let zv = zadic_variety!(5, 3, [[1, 2, 3], [4, 3, 2]]);
        assert_eq!(AdicVariety::new(5, vec![
            zadic_approx!(5, 3, [1, 2, 3]),
            zadic_approx!(5, 3, [4, 3, 2])
        ]), zv);
        let q = qadic!(uadic!(5, [1, 2, 3, 4]), -2);
        assert_eq!(QAdic::new(UAdic::new(5, vec![1, 2, 3, 4]), crate::AdicValuation::Finite(-2)), q);
        let iap = iadic_poly!(7, [-2, 0, 1]);
        assert_eq!(AdicPolynomial::new(7, vec![
            IAdic::new_neg(7, vec![5]),
            IAdic::zero(7),
            IAdic::one(7),
        ]), iap);
        let ap = adic_poly!(7, [iadic_neg!(7, [5]), iadic_pos!(7, []), iadic_pos!(7, [1]),]);
        assert_eq!(AdicPolynomial::new(7, vec![
            IAdic::new_neg(7, vec![5]),
            IAdic::zero(7),
            IAdic::one(7),
        ]), ap);
    }

}
