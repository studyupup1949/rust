//! Common tests among the adics

mod test_panics {
    use crate::{iadic_pos, radic, uadic, zadic_approx};

    #[test]
    #[should_panic(expected="6 is not prime")]
    fn u_nonprime() {
        let _ = uadic!(6, [2]);
    }
    #[test]
    #[should_panic(expected="6 is not prime")]
    fn i_nonprime() {
        let _ = iadic_pos!(6, [2]);
    }
    #[test]
    #[should_panic(expected="6 is not prime")]
    fn r_nonprime() {
        let _ = radic!(6, [2], [1]);
    }
    #[test]
    #[should_panic(expected="6 is not prime")]
    fn z_nonprime() {
        let _ = zadic_approx!(6, 6, [2]);
    }

    #[test]
    #[should_panic(expected="MixedCharacteristic")]
    fn u_mixed() {
        let _ = uadic!(5, [2]) + uadic!(7, [2]);
    }
    #[test]
    #[should_panic(expected="MixedCharacteristic")]
    fn i_mixed() {
        let _ = iadic_pos!(5, [2]) + iadic_pos!(7, [2]);
    }
    #[test]
    #[should_panic(expected="MixedCharacteristic")]
    fn r_mixed() {
        let _ = radic!(5, [2], [1]) + radic!(7, [2], [1]);
    }
    #[test]
    #[should_panic(expected="MixedCharacteristic")]
    fn z_mixed() {
        let _ = zadic_approx!(5, 6, [2]) + zadic_approx!(7, 6, [2]);
    }

}
