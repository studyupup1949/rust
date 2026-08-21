//! Common tests among the adics

use assertables::assert_approx_eq;

use crate::traits::HasDigits;

mod test_panics {

    #[test]
    #[should_panic(expected="6 is not prime")]
    fn u_nonprime() {
        let _ = uadic!(6, [2]);
    }
    #[test]
    #[should_panic(expected="6 is not prime")]
    fn i_nonprime() {
        let _ = eadic!(6, [2]);
    }
    #[test]
    #[should_panic(expected="6 is not prime")]
    fn r_nonprime() {
        let _ = eadic_rep!(6, [2], [1]);
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
        let _ = eadic!(5, [2]) + eadic!(7, [2]);
    }
    #[test]
    #[should_panic(expected="MixedCharacteristic")]
    fn r_mixed() {
        let _ = eadic_rep!(5, [2], [1]) + eadic_rep!(7, [2], [1]);
    }
    #[test]
    #[should_panic(expected="MixedCharacteristic")]
    fn z_mixed() {
        let _ = zadic_approx!(5, 6, [2]) + zadic_approx!(7, 6, [2]);
    }

}

#[test]
fn real_projection() {
    // 0._5 => 0
    assert_approx_eq!(eadic!(5, []).real_projection().unwrap(), 0.0);
    // ...444444._5 => 0.444444..._5 => 1
    assert_approx_eq!(eadic_rep!(5, [], [4]).real_projection().unwrap(), 1.0);
    // ...666666._7 => 1
    assert_approx_eq!(eadic_rep!(7, [], [6]).real_projection().unwrap(), 1.0);
    // 10._5 => 0.01_5 => 0.04
    assert_approx_eq!(eadic!(5, [0, 1]).real_projection().unwrap(), 0.04);
}
