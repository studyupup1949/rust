//! Example for finding the roots of different adic numbers

use adic::{RAdic, UAdic, variety_to_digits};
use num::Rational32;


/// Main method of example
pub fn main() {

    // 7-adic sqrt(2): two varieties

    let (n, a, p, precision) = (2, 2, 7, 6);
    println!("Calculating the {n}-root of {a} in the {p}-adics to {precision} digits");

    let a_digits = UAdic::new(p, vec![2]);
    assert_eq!(a_digits.integer_value(), a);
    let sqrt_two_digits = variety_to_digits(p, a_digits.integer_value() as i32, n, precision).unwrap();

    println!("{} solutions: {:?}", sqrt_two_digits.len(), sqrt_two_digits);

    assert_eq!(sqrt_two_digits.len(), 2);
    let sqrt_two_approx1 = UAdic::new(p, sqrt_two_digits[0].clone());
    let sqrt_two_approx2 = UAdic::new(p, sqrt_two_digits[1].clone());
    assert_eq!(sqrt_two_approx1, UAdic::new(p, vec![3, 1, 2, 6, 1, 2]));
    assert_eq!(sqrt_two_approx2, UAdic::new(p, vec![4, 5, 4, 0, 5, 4]));


    // 5-adic sqrt(2): no varieties

    let (n, a, p, precision) = (2, 2, 5, 6);
    println!("Calculating the {n}-root of {a} in the {p}-adics to {precision} digits");

    let a_digits = UAdic::new(p, vec![2]);
    assert_eq!(a_digits.integer_value(), a);
    let sqrt_two_digits = variety_to_digits(p, a_digits.integer_value() as i32, n, precision).unwrap();

    println!("{} solutions: {:?}", sqrt_two_digits.len(), sqrt_two_digits);

    assert_eq!(sqrt_two_digits.len(), 0);


    // 5-adic root_4(-1/4): four varieties

    let (n, a, p, precision) = (4, Rational32::new(-1, 4), 5, 6);
    println!("Calculating the {n}-root of {a} in the {p}-adics to {precision} digits");

    let a_digits = RAdic::new(p, vec![], vec![1]);
    assert_eq!(a_digits.rational_value(), a);
    assert_eq!(a_digits.truncate(precision).integer_value(), 3906);
    let root_a_digits = variety_to_digits(p, a_digits.truncate(precision).integer_value() as i32, n, precision as u32).unwrap();

    println!("{} solutions: {:?}", root_a_digits.len(), root_a_digits);
    let root_a_approx_solutions = root_a_digits.into_iter().map(
      |sol| UAdic::new(p, sol)
    ).collect::<Vec<_>>();
    assert_eq!(root_a_approx_solutions[0], UAdic::new(p, vec![1, 4, 3, 1, 3, 2]));
    assert_eq!(root_a_approx_solutions[1], UAdic::new(p, vec![2, 4, 3, 1, 3, 2]));
    assert_eq!(root_a_approx_solutions[2], UAdic::new(p, vec![3, 0, 1, 3, 1, 2]));
    assert_eq!(root_a_approx_solutions[3], UAdic::new(p, vec![4, 0, 1, 3, 1, 2]));

    println!("More digits, more precision:");
    let precision = 10;
    assert_eq!(a_digits.truncate(precision).integer_value(), 2441406);
    let root_a_digits = variety_to_digits(p, a_digits.truncate(precision).integer_value() as i32, n, precision as u32).unwrap();

    println!("{} solutions: {:?}", root_a_digits.len(), root_a_digits);
    let root_a_approx_solutions = root_a_digits.into_iter().map(
      |sol| UAdic::new(p, sol)
    ).collect::<Vec<_>>();
    assert_eq!(root_a_approx_solutions[0], UAdic::new(p, vec![1, 4, 3, 1, 3, 2, 3, 0, 2, 3]));
    assert_eq!(root_a_approx_solutions[1], UAdic::new(p, vec![2, 4, 3, 1, 3, 2, 3, 0, 2, 3]));
    assert_eq!(root_a_approx_solutions[2], UAdic::new(p, vec![3, 0, 1, 3, 1, 2, 1, 4, 2, 1]));
    assert_eq!(root_a_approx_solutions[3], UAdic::new(p, vec![4, 0, 1, 3, 1, 2, 1, 4, 2, 1]));

}
