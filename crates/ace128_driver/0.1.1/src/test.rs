use super::*;
use embedded_hal_mock::pin::{Transaction as PinTransaction, Mock as PinMock, State as PinState};

#[test]
fn correct_number_of_constants() {
    let count = ACE128_MAP.iter().filter_map(|&x| x).count();
    assert_eq!(count, 128);
}

#[test]
fn correct_sum_of_constants() {
    let sum: u32 = ACE128_MAP.iter().filter_map(|&x| x).map(|x| u32::from(x)).sum();
    assert_eq!(sum, 127 * 128 / 2);
}

#[test]
fn monotonically_increasing_constants() {
    let mut map = ACE128_MAP.clone();
    map.sort();
    // valid only if ix remains less than 256, else it overflows u8
    for (ix, val) in map.iter().filter_map(|&x| x).enumerate() {
        assert_eq!(ix as u8, val);
    }

}

#[test]
fn test_position_zero() {

    let expectations = [
        [PinTransaction::get(PinState::High)],
        [PinTransaction::get(PinState::High)],
        [PinTransaction::get(PinState::High)],
        [PinTransaction::get(PinState::High)],
        [PinTransaction::get(PinState::High)],
        [PinTransaction::get(PinState::High)],
        [PinTransaction::get(PinState::High)],
        [PinTransaction::get(PinState::Low)],
    ];

    let encoder = Ace128::new(
        PinMock::new(&expectations[0]),
        PinMock::new(&expectations[1]),
        PinMock::new(&expectations[2]),
        PinMock::new(&expectations[3]),
        PinMock::new(&expectations[4]),
        PinMock::new(&expectations[5]),
        PinMock::new(&expectations[6]),
        PinMock::new(&expectations[7]),
    );

    assert_eq!(encoder.read().unwrap(), Some(0));

}
