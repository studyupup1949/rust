//! Adic macros

#[macro_export]
/// Create a [UAdic](crate::UAdic) number more concisely
macro_rules! uadic {
    ( $p:expr, [$( $fixed_digits:expr ),*] ) => {
        $crate::UAdic::new($p, vec![$($fixed_digits,)*])
    };
}


#[macro_export]
/// Create a [RAdic](crate::RAdic) number more concisely
macro_rules! radic {
    ( $p:expr, [$( $fixed_digits:expr ),*], [$( $repeating_digits:expr ),*] ) => {
        $crate::RAdic::new($p, vec![$($fixed_digits,)*], vec![$($repeating_digits,)*])
    };
}


#[cfg(test)]
mod test {

    use crate::{RAdic, UAdic};
    use num::Rational32;

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
    }

}
