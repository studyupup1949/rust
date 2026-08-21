use adic::EAdic;
use super::{ClockCanvas, ClockMovement};

use crate::{
    draw::test_util::assert_diff_lt,
    shape::AdicCanvas,
};


#[test]
fn correct_numbers() {

    let adic_data = EAdic::new(5, vec![3, 2, 4, 1, 4, 1, 2]);
    let clock_canvas = ClockCanvas::builder().base(5).depth(6).build();
    let clock_shape = clock_canvas.draw_integer(&adic_data).unwrap();

    assert_eq!(5, clock_shape.base());
    assert_eq!(6, clock_shape.num_hands());

    let hands = clock_shape.hands();
    assert_eq!(0, hands[0].order);
    assert_eq!(3, hands[0].tick);
    assert_eq!(1, hands[1].order);
    assert_eq!(2, hands[1].tick);
    assert_eq!(2, hands[2].order);
    assert_eq!(4, hands[2].tick);
    assert_eq!(3, hands[3].order);
    assert_eq!(1, hands[3].tick);
    assert_eq!(4, hands[4].order);
    assert_eq!(4, hands[4].tick);
    assert_eq!(5, hands[5].order);
    assert_eq!(1, hands[5].tick);

}

#[test]
fn correct_bounds() {

    let adic_data = EAdic::new(5, vec![3, 2, 4, 1, 4, 1, 2]);
    let clock_canvas = ClockCanvas::builder().base(5).depth(6).clock_movement(ClockMovement::Ticking).build();
    let clock_shape = clock_canvas.draw_integer(&adic_data).unwrap();

    let hand_pos = clock_shape.hand_positions();
    assert!(0. <= hand_pos.iter().map(|hand| hand.head_position.0).min_by(f64::total_cmp).unwrap());
    assert!(100. >= hand_pos.iter().map(|hand| hand.head_position.0).max_by(f64::total_cmp).unwrap());
    assert!(0. <= hand_pos.iter().map(|hand| hand.head_position.1).min_by(f64::total_cmp).unwrap());
    assert!(100. >= hand_pos.iter().map(|hand| hand.head_position.1).max_by(f64::total_cmp).unwrap());
    assert!(0. <= hand_pos.iter().map(|hand| hand.radius).min_by(f64::total_cmp).unwrap());
    assert!(50. >= hand_pos.iter().map(|hand| hand.radius).max_by(f64::total_cmp).unwrap());

}

#[test]
fn correct_offsets() {

    let adic_data = EAdic::new(5, vec![3, 2, 4, 1, 4, 1, 2]);
    let clock_canvas = ClockCanvas::builder().base(5).depth(6).clock_movement(ClockMovement::Ticking).build();
    let clock_shape = clock_canvas.draw_integer(&adic_data).unwrap();

    for hand in clock_shape.hands() {
        assert_diff_lt!(0., hand.offset, 0.1);
    }

    let clock_canvas = ClockCanvas::builder().base(5).depth(6).clock_movement(ClockMovement::Sweeping).build();
    let clock_shape = clock_canvas.draw_integer(&adic_data).unwrap();

    let hands = clock_shape.hands();
    assert_diff_lt!(0., hands[0].offset, 0.1);
    assert_diff_lt!(0.6, hands[1].offset, 0.1);
    assert_diff_lt!(0.52, hands[2].offset, 0.1);
    assert_diff_lt!(0.90, hands[3].offset, 0.1);
    assert_diff_lt!(0.38, hands[4].offset, 0.1);
    assert_diff_lt!(0.88, hands[5].offset, 0.1);


}
