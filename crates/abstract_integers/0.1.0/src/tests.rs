use crate::*;

define_abstract_integer_modular!(SmallModular, 1, BigUint::from(255u32));

#[test]
fn wrapping() {
    let x1 = SmallModular::from_literal(254);
    let x2 = SmallModular::from_literal(3);
    let x3 = x1 + x2;
    assert_eq!(BigUint::from(2u32), x3.into());
    let x4 = SmallModular::from_literal(5);
    let x5 = x3 - x4;
    assert_eq!(BigUint::from(252u32), x5.into());
    let x6 = x5 / SmallModular::from_literal(4);
    assert_eq!(BigUint::from(63u32), x6.into());
}

define_abstract_integer_checked!(SmallBounded, 2, BigUint::from(0xFFFFu16));

#[test]
#[should_panic]
fn bounded() {
    let y1 = SmallBounded::from_literal(65530);
    let y2 = SmallBounded::from_literal(6);
    let _y3 = y1 + y2;
}

define_abstract_integer_checked!(Felem, 40, BigUint::from(1u32).shl(255) - BigUint::from(19u32));

#[test]
fn arith() {
    let x1 = Felem::from_literal(24875808327634644);
    let x2 = Felem::from_literal(91987276365379830);
    let x3 = x1 + x2;
    assert_eq!(BigUint::from(116863084693014474u128), x3.into())
}
