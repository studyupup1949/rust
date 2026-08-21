use assertables::assert_approx_eq;
use num::rational::Ratio;

use crate::{
    normed::{Normed, UltraNormed, Valuation},
    traits::{AdicPrimitive, HasApproximateDigits, HasDigitDisplay, HasDigits, PrimedFrom},
    UAdic, ZAdic,
};
use crate::num_adic::test_util::qu::*;
use super::QAdic;


#[test]
fn adjusts_validation() {
    assert_eq!(qadic!(uadic!(5, [2]), 5), qadic!(uadic!(5, [0, 0, 2]), 3));
    assert_eq!(qadic!(uadic!(5, [2]), -3), qadic!(uadic!(5, [0, 0, 2]), -5));
    assert_eq!(one(), five_fifth());
}

#[test]
fn adjusts_certainty() {
    assert_eq!(qadic!(zadic_approx!(5, 2, [2, 0]), 5), qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), 3));
    assert_eq!(qadic!(zadic_approx!(5, 2, [2, 0]), -3), qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), -5));
    assert_eq!(Valuation::Finite(7), qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), 3).certainty());
    assert_eq!(Valuation::Finite(2), qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), 3).significance());
}

#[test]
fn min_index() {
    assert_eq!(Valuation::Finite(0), qadic!(uadic!(5, [2]), 0).min_index());
    assert_eq!(Valuation::Finite(0), qadic!(uadic!(5, [2]), 2).min_index());
    assert_eq!(Valuation::Finite(-2), qadic!(uadic!(5, [2]), -2).min_index());
}

#[test]
fn empty_qz_adic() {

    assert_eq!(
        (None, Valuation::Finite(0)),
        QAdic::empty(5, 0).into_unit_and_valuation()
    );
    assert_eq!(
        (None, Valuation::Finite(1)),
        QAdic::empty(5, 1).into_unit_and_valuation()
    );
    assert_eq!(
        (None, Valuation::Finite(-1)),
        QAdic::new(ZAdic::empty(5), -1).into_unit_and_valuation()
    );
    assert_eq!(
        (None, Valuation::Finite(1)),
        qadic!(zadic_approx!(5, 2, [0, 0]), -1).into_unit_and_valuation()
    );

}

#[test]
fn display() {

    // UAdic
    assert_eq!("0._5", qadic!(uadic!(5, []), 0).to_string());
    assert_eq!("0._5", qadic!(uadic!(5, []), 1).to_string());
    assert_eq!("1._5", QAdic::<UAdic>::one(5).to_string());
    assert_eq!("2._5", qadic!(uadic!(5, [2]), 0).to_string());
    assert_eq!("20._5", qadic!(uadic!(5, [2]), 1).to_string());
    assert_eq!("0.2_5", qadic!(uadic!(5, [2]), -1).to_string());
    assert_eq!("100._5", qadic!(uadic!(5, [0, 1]), 1).to_string());
    assert_eq!("11._5", QAdic::<UAdic>::primed_from(5, 6).to_string());
    assert_eq!("220._5", qadic!(uadic!(5, [2, 2, 0, 0]), 1).to_string());
    assert_eq!("3.2_5", qadic!(uadic!(5, [2, 3]), -1).to_string());
    assert_eq!("0.032_5", qadic!(uadic!(5, [2, 3]), -3).to_string());
    assert_eq!("uk.a1_31", qadic!(uadic!(31, [1, 10, 20, 30]), -2).to_string());
    assert_eq!("[30][20].[10][1]_37", qadic!(uadic!(37, [1, 10, 20, 30]), -2).to_string());

    // IAdic
    assert_eq!("(4)._5", qadic!(eadic_neg!(5, []), 0).to_string());
    assert_eq!("(4)3._5", qadic!(eadic_neg!(5, [3]), 0).to_string());
    assert_eq!("(4)30._5", qadic!(eadic_neg!(5, [3]), 1).to_string());
    assert_eq!("(4).3_5", qadic!(eadic_neg!(5, [3]), -1).to_string());
    assert_eq!("(4).443_5", qadic!(eadic_neg!(5, [3]), -3).to_string());
    assert_eq!("(u)k.a1_31", qadic!(eadic_neg!(31, [1, 10, 20]), -2).to_string());
    assert_eq!("([36])[20].[10][1]_37", qadic!(eadic_neg!(37, [1, 10, 20]), -2).to_string());

    // RAdic
    assert_eq!("(4)._5", qadic!(eadic_rep!(5, [], [4]), 0).to_string());
    assert_eq!("(01)._5", qadic!(eadic_rep!(5, [], [1, 0]), 0).to_string());
    assert_eq!("(10)._5", qadic!(eadic_rep!(5, [], [1, 0]), 1).to_string());
    assert_eq!("(10)00._5", qadic!(eadic_rep!(5, [], [1, 0]), 3).to_string());
    assert_eq!("(10).1_5", qadic!(eadic_rep!(5, [], [1, 0]), -1).to_string());
    assert_eq!("(01).0101_5", qadic!(eadic_rep!(5, [], [1, 0]), -4).to_string());
    assert_eq!("(uk)a.1_31", qadic!(eadic_rep!(31, [1, 10], [20, 30]), -1).to_string());
    assert_eq!("([30][20])[10].[1]_37", qadic!(eadic_rep!(37, [1, 10], [20, 30]), -1).to_string());

    // ZAdic
    assert_eq!("20._5", qadic!(ZAdic::from(uadic!(5, [2])), 1).to_string());
    assert_eq!("...0000._5", qadic!(zadic_approx!(5, 4, []), 0).to_string());
    assert_eq!("...00000._5", qadic!(zadic_approx!(5, 4, []), 1).to_string());
    assert_eq!("...000._5", qadic!(zadic_approx!(5, 4, []), -1).to_string());
    assert_eq!("...0001._5", qadic!(zadic_approx!(5, 4, [1]), 0).to_string());
    assert_eq!("...00010._5", qadic!(zadic_approx!(5, 4, [1]), 1).to_string());
    assert_eq!("...00.01_5", qadic!(zadic_approx!(5, 4, [1]), -2).to_string());
    assert_eq!("...6213._7", qadic!(zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2]), 0).to_string());
    assert_eq!("...621300._7", qadic!(zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2]), 2).to_string());
    assert_eq!("...62.13_7", qadic!(zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2]), -2).to_string());
    assert_eq!("...uk.a1_31", qadic!(zadic_approx!(31, 4, [1, 10, 20, 30]), -2).to_string());
    assert_eq!("...[30][20].[10][1]_37", qadic!(zadic_approx!(37, 4, [1, 10, 20, 30]), -2).to_string());

    // QAdic::digit_display
    assert_eq!(("".to_string(), "32100".to_string()), qadic!(uadic!(5, [1, 2, 3]), 2).digit_display());
    assert_eq!(("1".to_string(), "(4)3".to_string()), qadic!(eadic_neg!(5, [1, 3]), -1).digit_display());
    assert_eq!(("42431".to_string(), "(42)".to_string()), qadic!(eadic_rep!(5, [1, 3], [4, 2]), -5).digit_display());
    assert_eq!(("".to_string(), "...341200".to_string()), qadic!(zadic_approx!(5, 4, [2, 1, 4, 3]), 2).digit_display());

}

#[test]
fn normed() {

    let a = QAdic::from_integer(eadic!(5, [1, 2, 3]));
    assert!(a.is_unit());
    assert_eq!(Ratio::from(1), a.norm());
    assert_eq!(Valuation::Finite(0), a.valuation());
    assert_eq!(Some(eadic!(5, [1, 2, 3])), a.unit());

    let a = QAdic::from_integer(eadic!(5, [0, 1, 2]));
    assert!(!a.is_unit());
    assert_eq!(Ratio::new(1, 5), a.norm());
    assert_eq!(Valuation::Finite(1), a.valuation());
    assert_eq!(Some(eadic!(5, [1, 2])), a.unit());

    let a = QAdic::new(eadic!(5, [1, 2, 3]), -1);
    assert!(!a.is_unit());
    assert_eq!(Ratio::from(5), a.norm());
    assert_eq!(Valuation::Finite(-1), a.valuation());
    assert_eq!(Some(eadic!(5, [1, 2, 3])), a.unit());

    let a = QAdic::new(ZAdic::empty(5), -1);
    assert!(!a.is_unit());
    assert_eq!(Ratio::from(5), a.norm());
    assert_eq!(Valuation::Finite(-1), a.valuation());
    assert_eq!(None, a.unit());

}

#[test]
fn real_projection() {
    // 0._5 => 0
    assert_approx_eq!(qadic!(eadic!(5, []), 0).real_projection().unwrap(), 0.0);
    // ...444444.4_5 => 4.44444...._5 => 5
    assert_approx_eq!(qadic!(eadic_rep!(5, [], [4]), -1).real_projection().unwrap(), 5.0);
    // ...666666.66_7 => 66.66666..._7 => 49
    assert_approx_eq!(qadic!(eadic_rep!(7, [], [6]), -2).real_projection().unwrap(), 49.0);
    // 1000._5 => 0.0001_5 => 0.0016
    assert_approx_eq!(qadic!(eadic!(5, [0, 1]), 2).real_projection().unwrap(), 0.0016);
}
