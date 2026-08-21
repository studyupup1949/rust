use address_cmp::*;
use std::hash::Hash;
use std::{collections::hash_map::RandomState, hash::BuildHasher};

#[test]
fn same_struct_is_equal() {
    #[derive(AddressEq, Debug)]
    struct A {
        pub a: u8,
    }

    let a = A { a: 0 };

    assert_eq!(a, a);
}

#[test]
fn same_struct_with_lifetime_is_equal() {
    #[derive(AddressEq, Debug)]
    struct A<'a> {
        pub a: &'a u8,
    }

    let num = 0;
    let a = A { a: &num };

    assert_eq!(a, a);
}

#[test]
fn same_empty_struct_is_equal() {
    #[derive(AddressEq, Debug)]
    struct A {}

    let a = A {};

    assert_eq!(a, a);
}

#[test]
fn different_struct_is_not_equal() {
    #[derive(AddressEq, Debug)]
    struct A {
        pub a: u8,
    }

    let a1 = A { a: 0 };
    let a2 = A { a: 0 };

    assert_ne!(a1, a2);
}

#[test]
fn different_struct_with_lifetime_is_not_equal() {
    #[derive(AddressEq, Debug)]
    struct A<'a> {
        pub a: &'a u8,
    }

    let num = 0;

    let a1 = A { a: &num };
    let a2 = A { a: &num };

    assert_ne!(a1, a2);
}

#[test]
fn different_empty_struct_is_not_equal() {
    #[derive(AddressEq, Debug)]
    struct A {}

    let a1 = A {};
    let a2 = A {};

    assert_ne!(a1, a2);
}

#[test]
fn same_struct_is_hashed_the_same() {
    #[derive(AddressHash)]
    struct A {
        pub a: u8,
    }

    let a = A { a: 0 };
    let mut state = RandomState::new().build_hasher();
    let hash1 = a.hash(&mut state);
    let hash2 = a.hash(&mut state);

    assert_eq!(hash1, hash2);
}

#[test]
fn different_struct_is_hashed_the_differently() {
    #[derive(AddressHash)]
    struct A {
        pub a: u8,
    }

    let a1 = A { a: 0 };
    let a2 = A { a: 0 };
    let mut state = RandomState::new().build_hasher();
    let hash1 = a1.hash(&mut state);
    let hash2 = a2.hash(&mut state);

    assert_eq!(hash1, hash2);
}

#[test]
fn same_empty_struct_is_hashed_the_same() {
    #[derive(AddressHash)]
    struct A {}

    let a = A {};
    let mut state = RandomState::new().build_hasher();
    let hash1 = a.hash(&mut state);
    let hash2 = a.hash(&mut state);

    assert_eq!(hash1, hash2);
}

#[test]
fn different_empty_struct_is_hashed_the_differently() {
    #[derive(AddressHash)]
    struct A {}

    let a1 = A {};
    let a2 = A {};
    let mut state = RandomState::new().build_hasher();
    let hash1 = a1.hash(&mut state);
    let hash2 = a2.hash(&mut state);

    assert_eq!(hash1, hash2);
}

#[test]
fn same_struct_is_ordered_same() {
    #[derive(AddressOrd, AddressEq)]
    struct A {
        pub a: u8,
    }

    let a = A { a: 0 };

    assert!(!(a < a));
    assert!(!(a > a));
    assert!((a <= a));
    assert!((a >= a));
}

#[test]
fn same_empty_struct_is_ordered_same() {
    #[derive(AddressOrd, AddressEq)]
    struct A {}

    let a = A {};

    assert!(!(a < a));
    assert!(!(a > a));
    assert!((a <= a));
    assert!((a >= a));
}

#[test]
fn different_struct_is_ordered_differently() {
    #[derive(AddressOrd, AddressEq)]
    struct A {
        pub a: u8,
    }

    let a1 = A { a: 0 };
    let a2 = A { a: 0 };

    assert!((a1 < a2) || (a2 < a1));
}

#[test]
fn different_empty_struct_is_ordered_differently() {
    #[derive(AddressEq, AddressOrd)]
    struct A {}

    let a1 = A {};
    let a2 = A {};

    assert!((a1 < a2) || (a2 < a1));
}
