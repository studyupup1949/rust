use crate::{
    num_adic::{PowAdic, MAdic},
    traits::{AdicPrimitive, PrimedFrom},
    EAdic, UAdic, ZAdic,
};
use crate::num_adic::test_util::{z, qe, qu, qz};
use super::{LocalOne, LocalZero};


#[test]
fn local_zero() {

    // UAdic
    let a = UAdic::new(5, vec![0, 1, 2, 3, 4]);
    let b = a.local_zero();
    assert_eq!(a.clone() + b, a);

    // EAdic
    assert_eq!(EAdic::zero(5), EAdic::primed_from(5, 5).local_zero());
    assert_ne!(EAdic::zero(5), EAdic::primed_from(7, -3).local_zero());

    // ZAdic
    assert_eq!(z::zero_e(), z::five_4().local_zero());
    assert_ne!(z::zero_e(), zadic_approx!(7, 4, [3]).local_zero());

    // QAdic
    assert_eq!(qu::zero(), qadic!(uadic!(5, [2, 3]), -3).local_zero());
    assert_eq!(qe::zero(), qadic!(eadic_neg!(5, [3]), -3).local_zero());
    assert_eq!(qe::zero(), qadic!(eadic_rep!(5, [], [1, 0]), -4).local_zero());
    assert_eq!(qz::zero_e(), qadic!(zadic_approx!(5, 4, [1]), -2).local_zero());

    assert_ne!(qu::zero(), qadic!(uadic!(7, [2, 3]), -3).local_zero());
    assert_ne!(qe::zero(), qadic!(eadic_neg!(7, [3]), -3).local_zero());
    assert_ne!(qe::zero(), qadic!(eadic_rep!(7, [], [1, 0]), -4).local_zero());
    assert_ne!(qz::zero_e(), qadic!(zadic_approx!(7, 4, [1]), -2).local_zero());

    // MAdic
    let ac = MAdic::new([
        PowAdic::new(qadic!(zadic_approx!(2, 30, [1]), -5), 1),
        PowAdic::new(qadic!(zadic_approx!(5, 12, [2]), -2), 1),
    ]);
    let ac_zero = MAdic::new([
        PowAdic::new(qadic!(ZAdic::from(uadic!(2, [])), 0), 1),
        PowAdic::new(qadic!(ZAdic::from(uadic!(5, [])), 0), 1),
    ]);
    assert_eq!(ac_zero, ac.local_zero() );

    // PowAdic
    assert_eq!(
        PowAdic::new(zadic_approx!(5, 7, [0, 1, 2, 3, 4, 0, 1]), 2).local_zero(),
        PowAdic::new(ZAdic::zero(5), 2)
    );

}

#[test]
fn local_one() {

    // UAdic
    let a = UAdic::new(5, vec![0, 1, 2, 3, 4]);
    let b = a.local_one();
    assert_eq!(a.clone() * b, a);

    // EAdic
    assert_eq!(EAdic::one(5), EAdic::primed_from(5, 5).local_one());
    assert_ne!(EAdic::one(5), EAdic::primed_from(7, -3).local_one());

    // ZAdic
    assert_eq!(z::one_e(), z::five_4().local_one());
    assert_ne!(z::one_e(), zadic_approx!(7, 4, [3]).local_one());

    // QAdic
    assert_eq!(qu::one(), qadic!(uadic!(5, [2, 3]), -3).local_one());
    assert_eq!(qe::one(), qadic!(eadic_neg!(5, [3]), -3).local_one());
    assert_eq!(qe::one(), qadic!(eadic_rep!(5, [], [1, 0]), -4).local_one());
    assert_eq!(qz::one_e(), qadic!(zadic_approx!(5, 4, [1]), -2).local_one());

    assert_ne!(qu::one(), qadic!(uadic!(7, [2, 3]), -3).local_one());
    assert_ne!(qe::one(), qadic!(eadic_neg!(7, [3]), -3).local_one());
    assert_ne!(qe::one(), qadic!(eadic_rep!(7, [], [1, 0]), -4).local_one());
    assert_ne!(qz::one_e(), qadic!(zadic_approx!(7, 4, [1]), -2).local_one());

    // MAdic
    let ac = MAdic::new([
        PowAdic::new(qadic!(zadic_approx!(2, 30, [1]), -5), 1),
        PowAdic::new(qadic!(zadic_approx!(5, 12, [2]), -2), 1),
    ]);
    let ac_one = MAdic::new([
        PowAdic::new(qadic!(ZAdic::one(2), 0), 1),
        PowAdic::new(qadic!(ZAdic::one(5), 0), 1),
    ]);
    assert_eq!(ac_one, ac.local_one() );

    // PowAdic
    assert_eq!(
        PowAdic::new(zadic_approx!(5, 7, [0, 1, 2, 3, 4, 0, 1]), 2).local_one(),
        PowAdic::new(ZAdic::one(5), 2)
    );

}
