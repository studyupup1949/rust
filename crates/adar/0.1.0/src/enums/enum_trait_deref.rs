// Note: This file contains tests for the EnumTraitDeref macro.

#[cfg(test)]
mod test {
    use adar_macros::*;
    trait TestTrait {
        fn my_func(&self) -> bool;
        fn my_mut_func(&self) -> bool;
    }
    struct A;
    impl TestTrait for A {
        fn my_func(&self) -> bool {
            true
        }

        fn my_mut_func(&self) -> bool {
            true
        }
    }
    struct B;
    impl TestTrait for B {
        fn my_func(&self) -> bool {
            false
        }

        fn my_mut_func(&self) -> bool {
            false
        }
    }

    #[EnumTraitDeref(TestTrait)]
    enum TestEnumTraitDeref {
        A(A),
        B(B),
    }

    #[EnumTraitDerefMut(TestTrait)]
    enum TestEnumTraitDerefMut {
        A(A),
        B(B),
    }

    #[test]
    fn test_enum_trait_deref() {
        assert!(TestEnumTraitDeref::A(A).my_func());
        assert!(!TestEnumTraitDeref::B(B).my_func());
    }
    #[test]
    fn test_enum_trait_deref_mut() {
        assert!(TestEnumTraitDerefMut::A(A).my_func());
        assert!(!TestEnumTraitDerefMut::B(B).my_func());
        assert!(TestEnumTraitDerefMut::A(A).my_mut_func());
        assert!(!TestEnumTraitDerefMut::B(B).my_mut_func());
    }
}
