mod util;

use address_cmp::*;
use util::calculate_hash;

#[test]
fn same_enum_is_equal() {
    #[derive(AddressEq, Debug)]
    enum A {
        B,
    }

    let a = A::B;

    assert_eq!(a, a);
}

#[test]
fn different_enum_is_not_equal() {
    #[derive(AddressEq, Debug)]
    enum A {
        B,
    }

    let a1 = A::B;
    let a2 = A::B;

    assert_ne!(a1, a2);
}

#[test]
fn same_enum_is_hashed_the_same() {
    #[derive(AddressHash)]
    enum A {
        B,
    }

    let a = A::B;
    let hash1 = calculate_hash(&a);
    let hash2 = calculate_hash(&a);

    assert_eq!(hash1, hash2);
}

#[test]
fn different_enum_is_hashed_differently() {
    #[derive(AddressHash)]
    enum A {
        B,
    }

    let a1 = A::B;
    let a2 = A::B;
    let hash1 = calculate_hash(&a1);
    let hash2 = calculate_hash(&a2);

    assert_ne!(hash1, hash2);
}

#[test]
fn same_enum_is_ordered_same() {
    #[derive(AddressOrd, AddressEq)]
    enum A {
        B,
    }

    let a = A::B;

    assert!(!(a < a));
    assert!(!(a > a));
    assert!((a <= a));
    assert!((a >= a));
}

#[test]
fn different_enum_is_ordered_differently() {
    #[derive(AddressOrd, AddressEq)]
    enum A {
        B,
    }

    let a1 = A::B;
    let a2 = A::B;

    assert!((a1 < a2) || (a2 < a1));
}
