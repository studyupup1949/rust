use address_cmp::*;
use std::hash::Hash;
use std::{collections::hash_map::RandomState, hash::BuildHasher};

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
    let mut state = RandomState::new().build_hasher();
    let hash1 = a.hash(&mut state);
    let hash2 = a.hash(&mut state);

    assert_eq!(hash1, hash2);
}

#[test]
fn different_enum_is_hashed_the_differently() {
    #[derive(AddressHash)]
    enum A {
        B,
    }

    let a1 = A::B;
    let a2 = A::B;
    let mut state = RandomState::new().build_hasher();
    let hash1 = a1.hash(&mut state);
    let hash2 = a2.hash(&mut state);

    assert_eq!(hash1, hash2);
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
