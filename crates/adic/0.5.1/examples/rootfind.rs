//! Example for finding the roots of different adic numbers

use adic::{
    traits::{AdicInteger, CanTruncate},
    EAdic, UAdic, ZAdic,
};
use num::Rational32;


/// Main method of example
pub fn main() {

    // 7-adic sqrt(2): two varieties

    let (n, a, p, precision) = (2, 2, 7, 6);
    println!("Calculating the {n}-root of {a} in the {p}-adics to {precision} digits");

    let u = UAdic::new(p, vec![2]);
    assert_eq!(u.u32_value(), Ok(a));
    let sqrt_two_var = u.nth_root(n, precision).unwrap();
    let sqrt_two_sols = sqrt_two_var.roots().collect::<Vec<_>>();

    println!("{} solutions: {}", sqrt_two_sols.len(), sqrt_two_var);

    assert_eq!(sqrt_two_sols.len(), 2);
    assert_eq!(sqrt_two_sols[0], &ZAdic::new_approx(7, 6, vec![3, 1, 2, 6, 1, 2]));
    assert_eq!(sqrt_two_sols[1], &ZAdic::new_approx(7, 6, vec![4, 5, 4, 0, 5, 4]));


    // 5-adic sqrt(2): no varieties

    let (n, a, p, precision) = (2, 2, 5, 6);
    println!("Calculating the {n}-root of {a} in the {p}-adics to {precision} digits");

    let u = UAdic::new(p, vec![2]);
    assert_eq!(u.u32_value(), Ok(a));
    let sqrt_two_var = u.nth_root(n, precision).unwrap();
    let sqrt_two_sols = sqrt_two_var.roots().collect::<Vec<_>>();

    println!("{} solutions: {}", sqrt_two_sols.len(), sqrt_two_var);

    assert_eq!(sqrt_two_sols.len(), 0);


    // 5-adic root_4(-1/4): four varieties

    let (n, a, p, precision) = (4, Rational32::new(-1, 4), 5, 6);
    println!("Calculating the {n}-root of {a} in the {p}-adics to {precision} digits");

    let r = EAdic::new_repeating(p, vec![], vec![1]);
    assert_eq!(r.rational_value(), Ok(a));
    assert_eq!(r.truncation(precision).u32_value(), Ok(3906));
    let root_a_var = r.nth_root(n, precision).unwrap();
    let root_a_sols = root_a_var.roots().collect::<Vec<_>>();

    println!("{} solutions: {}", root_a_sols.len(), root_a_var);
    assert_eq!(root_a_sols[0], &ZAdic::new_approx(p, 6, vec![1, 4, 3, 1, 3, 2]));
    assert_eq!(root_a_sols[1], &ZAdic::new_approx(p, 6, vec![2, 4, 3, 1, 3, 2]));
    assert_eq!(root_a_sols[2], &ZAdic::new_approx(p, 6, vec![3, 0, 1, 3, 1, 2]));
    assert_eq!(root_a_sols[3], &ZAdic::new_approx(p, 6, vec![4, 0, 1, 3, 1, 2]));

    println!("More digits, more precision:");
    let precision = 10;
    assert_eq!(r.truncation(precision).u32_value(), Ok(2_441_406));
    let root_a_var = r.nth_root(n, precision).unwrap();
    let root_a_sols = root_a_var.roots().collect::<Vec<_>>();

    println!("{} solutions: {}", root_a_sols.len(), root_a_var);
    assert_eq!(root_a_sols[0], &ZAdic::new_approx(p, 10, vec![1, 4, 3, 1, 3, 2, 3, 0, 2, 3]));
    assert_eq!(root_a_sols[1], &ZAdic::new_approx(p, 10, vec![2, 4, 3, 1, 3, 2, 3, 0, 2, 3]));
    assert_eq!(root_a_sols[2], &ZAdic::new_approx(p, 10, vec![3, 0, 1, 3, 1, 2, 1, 4, 2, 1]));
    assert_eq!(root_a_sols[3], &ZAdic::new_approx(p, 10, vec![4, 0, 1, 3, 1, 2, 1, 4, 2, 1]));

}
