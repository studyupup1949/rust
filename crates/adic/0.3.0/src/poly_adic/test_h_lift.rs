use std::iter::repeat_n;
use itertools::Itertools;
use num::{traits::Pow, Rational32};
use crate::{
    iadic_pos, radic, zadic_approx, zadic_variety,
    AdicError, AdicInteger, RAdic, UAdic, IAdic, ZAdic, ZAdicVariety
};

use crate::num_adic::test_util::{i, r};

use super::{nth_root, polynomial_variety, roots_of_unity};

#[test]
fn sqrt_2_digits() {

    // 2-adic
    let expected = ZAdicVariety::empty(2);
    let actual= nth_root(&i::two2(), 2, 3);
    assert_eq!(expected, actual.unwrap());
    let actual= polynomial_variety(2, &[-i::two2(), i::zero2(), i::one2()], 3);
    assert_eq!(expected, actual.unwrap());

    // 3-adic
    let expected = ZAdicVariety::empty(3);
    let actual= nth_root(&i::two3(), 2, 3);
    assert_eq!(expected, actual.unwrap());
    let actual= polynomial_variety(3, &[-i::two3(), i::zero3(), i::one3()], 3);
    assert_eq!(expected, actual.unwrap());

    // 5-adic
    let expected = ZAdicVariety::empty(5);
    let actual= nth_root(&i::two(), 2, 3);
    assert_eq!(expected, actual.unwrap());
    let actual= polynomial_variety(5, &[-i::two(), i::zero(), i::one()], 3);
    assert_eq!(expected, actual.unwrap());

    // 7-adic
    let expected = zadic_variety!(7, 3, [
        [3, 1, 2],
        [4, 5, 4],
    ]);
    let actual = nth_root(&i::two7(), 2, 3);
    assert_eq!(expected, actual.unwrap());
    let actual = polynomial_variety(7, &[-i::two7(), i::zero7(), i::one7()], 3);
    assert_eq!(expected, actual.unwrap());

    for zadic in expected.into_roots() {
        assert_eq!(zadic_approx!(7, 3, [2]), zadic.clone() * zadic.clone());
    }

}

#[test]
fn nth_root_of_p() {

    // 5-adic

    // sqrt(5^m)
    let expected = ZAdicVariety::empty(5);
    let actual = nth_root(&i::five(), 2, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(5, 4, [
        [0, 1, 0, 0],
        [0, 4, 4, 4],
    ]);
    let actual = nth_root(&i::twenty_five(), 2, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = ZAdicVariety::empty(5);
    let actual = nth_root(&i::one_twenty_five(), 2, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(5, 4, [
        [0, 0, 1, 0],
        [0, 0, 4, 4],
    ]);
    let actual = nth_root(&i::six_twenty_five(), 2, 4);
    assert_eq!(expected, actual.unwrap());

    // cubert(5^m)
    let expected = ZAdicVariety::empty(5);
    let actual = nth_root(&i::five(), 3, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = ZAdicVariety::empty(5);
    let actual = nth_root(&i::twenty_five(), 3, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(5, 4, [
        [0, 1, 0],
    ]);
    let actual = nth_root(&i::one_twenty_five(), 3, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = ZAdicVariety::empty(5);
    let actual = nth_root(&i::six_twenty_five(), 3, 4);
    assert_eq!(expected, actual.unwrap());

    // fourthrt(5^m)
    let expected = ZAdicVariety::empty(5);
    let actual = nth_root(&i::five(), 4, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = ZAdicVariety::empty(5);
    let actual = nth_root(&i::twenty_five(), 4, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = ZAdicVariety::empty(5);
    let actual = nth_root(&i::one_twenty_five(), 4, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(5, 4, [
        [0, 1, 0, 0],
        [0, 2, 1, 2],
        [0, 3, 3, 2],
        [0, 4, 4, 4],
    ]);
    let actual = nth_root(&i::six_twenty_five(), 4, 4);
    assert_eq!(expected, actual.unwrap());

    let expected = zadic_variety!(7, 6, [
        [0, 3, 1, 2, 6, 1],
        [0, 4, 5, 4, 0, 5],
    ]);
    let actual = nth_root(&i::ninety_eight7(), 2, 6);
    assert_eq!(expected, actual.unwrap());

    // 7-adic

    let expected = zadic_variety!(7, 5, [
        [0, 3, 1, 2, 6],
        [0, 4, 5, 4, 0],
    ]);
    let actual = nth_root(&i::ninety_eight7(), 2, 5);
    assert_eq!(actual.unwrap(), expected);

    let ninety_eight_5 = zadic_approx!(7, 6, [0, 0, 2]);
    for zadic in expected.into_roots() {
        assert_eq!(ninety_eight_5, zadic.clone() * zadic.clone());
    }

}

#[test]
fn pth_root() {

    // cubert(10) in 3-adics
    let solution = zadic_approx!(3, 8, [1, 1, 1, 0, 0, 0, 2, 1]);
    let expected = ZAdicVariety::new(3, vec![solution.clone()]);
    let actual = nth_root(&i::ten3(), 3, 8);
    assert_eq!(expected, actual.unwrap());
    assert_eq!(i::ten3(), IAdic::from(solution.pow(3).into_truncation_to_uadic().unwrap()));

    // fifthrt(1._5) has a single root, 1
    assert_eq!(
        zadic_variety!(5, 8, [[1]]),
        nth_root(&UAdic::one(5), 5, 8).unwrap()
    );

    // fifthrt(112._5) has a single root, 2
    let var2 = zadic_variety!(5, 8, [[2]]);
    assert_eq!(var2, nth_root(&iadic_pos!(5, [2, 1, 1]), 5, 8).unwrap());

    // fifthrt((12._5)^5) has one root: 12._5
    let seven = iadic_pos!(5, [2, 1]);
    let seven_to_fifth = iadic_pos!(5, [2, 1, 2, 4, 1, 0, 1]);
    assert_eq!(seven.clone().pow(5), seven_to_fifth);
    let expected = ZAdicVariety::new(5, vec![
        seven.into_approximation(4)
    ]);
    let actual = nth_root(
        &seven_to_fifth.into_approximation(20),
        5, 4
    );
    assert_eq!(expected, actual.unwrap());

    // nth-rt((-9/2)^n)
    let neg_nine_half = radic!(3, [0, 0], [1]);
    assert_eq!(Rational32::new(-9, 2), neg_nine_half.rational_value());
    let squared = neg_nine_half.clone().pow(2);
    assert_eq!(Rational32::new(81, 4), squared.rational_value());
    let cubed = neg_nine_half.clone().pow(3);
    assert_eq!(Rational32::new(-729, 8), cubed.rational_value());
    let fourthed = neg_nine_half.clone().pow(4);
    let fifthed = neg_nine_half.clone().pow(5);
    let zneg_nine_half = neg_nine_half.approximation(4);
    let zpos_nine_half = (-neg_nine_half).approximation(4);
    let var_one = ZAdicVariety::new(3, vec![zneg_nine_half.clone()]);
    let var_both = ZAdicVariety::new(3, vec![zneg_nine_half.clone(), zpos_nine_half.clone()]);
    assert_eq!(var_both, nth_root(&squared, 2, 4).unwrap());
    assert_eq!(var_one, nth_root(&cubed, 3, 4).unwrap());
    assert_eq!(var_both, nth_root(&fourthed, 4, 4).unwrap());
    assert_eq!(var_one, nth_root(&fifthed, 5, 4).unwrap());

}

#[test]
fn two_adic() {

    // sqrt(2^m)
    let expected = ZAdicVariety::empty(2);
    let actual = nth_root(&i::two2(), 2, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(2, 4, [
        [0, 1, 0, 0],
        [0, 1, 1, 1],
    ]);
    let actual = nth_root(&i::four2(), 2, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = ZAdicVariety::empty(2);
    let actual = nth_root(&i::eight2(), 2, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(2, 4, [
        [0, 0, 1, 0],
        [0, 0, 1, 1],
    ]);
    let actual = nth_root(&i::sixteen2(), 2, 4);
    assert_eq!(expected, actual.unwrap());

    // sqrt(3)
    let expected = ZAdicVariety::empty(2);
    let actual = nth_root(&i::three2(), 2, 4);
    assert_eq!(expected, actual.unwrap());

    // sqrt(17)
    let expected = zadic_variety!(2, 8, [
        [1, 0, 0, 1, 0, 1, 1, 1],
        [1, 1, 1, 0, 1, 0, 0, 0],
    ]);
    let actual = nth_root(&i::seventeen2(), 2, 8);
    assert_eq!(expected, actual.unwrap());

    let expected = zadic_variety!(2, 8, [
        [1, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1, 1, 1],
    ]);
    let actual = nth_root(&i::one2(), 2, 8).unwrap();
    assert_eq!(expected, actual);
    let actual = nth_root(&i::one2(), 4, 8).unwrap();
    assert_eq!(expected, actual);
    let actual = nth_root(&i::one2(), 6, 8).unwrap();
    assert_eq!(expected, actual);
    let actual = nth_root(&i::one2(), 8, 8).unwrap();
    assert_eq!(expected, actual);
    let actual = nth_root(&i::one2(), 10, 8).unwrap();
    assert_eq!(expected, actual);

    let expected = zadic_variety!(2, 8, [
        [1, 0, 1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 0, 0, 0],
    ]);
    let eighty_one = iadic_pos!(2, [1, 0, 0, 0, 1, 0, 1]);
    let actual = nth_root(&eighty_one, 4, 8).unwrap();
    assert_eq!(expected, actual);

}

#[test]
fn a_equals_zero() {
    // Returns a single solution
    let expected = ZAdicVariety::new(5, vec![ZAdic::zero(5)]);
    let actual = nth_root(&i::zero(), 2, 3);
    assert_eq!(actual.unwrap(), expected);
    let actual = polynomial_variety(5, &[i::zero(), i::zero(), i::one()], 3);
    assert_eq!(actual.unwrap(), expected);
}

#[test]
fn sqrt_fractions() {

    // 2-adic

    let pos_one_ninth = radic!(2, [1], [0, 0, 1, 1, 1, 0]);
    let neg_one_third = radic!(2, [], [1, 0]);
    let pos_one_third = radic!(2, [1], [1, 0]);
    let expected = ZAdicVariety::new(2, vec![
        neg_one_third.into_approximation(6),
        pos_one_third.into_approximation(6),
    ]);
    let actual = nth_root(&pos_one_ninth, 2, 6).unwrap();
    assert_eq!(expected, actual);

    // 5-adic

    let expected = ZAdicVariety::new(5, vec![
        r::neg_1_2().into_approximation(6),
        r::pos_1_2().into_approximation(6),
    ]);
    let actual = nth_root(&r::pos_1_4(), 2, 6).unwrap();
    assert_eq!(expected, actual);

    // 7-adic

    let pos_one_sixteenth = radic!(7, [4], [6, 3]);
    let neg_one_fourth = radic!(7, [], [5, 1]);
    let pos_one_fourth = radic!(7, [2], [5, 1]);
    let neg_one_half = radic!(7, [], [3]);
    let pos_one_half = radic!(7, [4], [3]);
    let expected = ZAdicVariety::new(7, vec![
        pos_one_fourth.approximation(6),
        neg_one_fourth.approximation(6),
    ]);
    let actual = nth_root(&pos_one_sixteenth, 2, 6).unwrap();
    assert_eq!(expected, actual);
    let expected = ZAdicVariety::new(7, vec![
        neg_one_half.into_approximation(6),
        pos_one_half.into_approximation(6),
    ]);
    let actual = nth_root(&pos_one_fourth, 2, 6).unwrap();
    assert_eq!(expected, actual);

}

#[test]
fn polynomial_variety_refactor() {
    let expected = zadic_variety!(7, 6, [
        [0, 3, 1, 2, 6, 1],
        [0, 4, 5, 4, 0, 5],
    ]);
    let ninety_eight = iadic_pos!(7, [0, 0, 2]);
    let actual = polynomial_variety(7, &[-ninety_eight.clone(), i::zero7(), i::one7()], 6);
    assert_eq!(expected, actual.unwrap());

    let actual = polynomial_variety(5, &[i::zero(), ninety_eight.clone(), i::zero(), i::one()], 3);
    assert!(matches!(actual, Err(AdicError::NotImplemented(_))));

}

#[test]
fn unity_roots() {

    let expected = zadic_variety!(2, 6, [
        [1, 0, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1],
    ]);
    let actual = roots_of_unity(2, 6).unwrap();
    assert_eq!(expected, actual);
    assert!(actual.roots().try_len() == Ok(2));
    assert!(actual.roots().contains(&zadic_approx![2, 6, [1, 0, 0, 0, 0, 0]]));
    assert!(actual.roots().contains(&zadic_approx![2, 6, [1, 1, 1, 1, 1, 1]]));

    let expected = zadic_variety!(3, 6, [
        [1, 0, 0, 0, 0, 0],
        [2, 2, 2, 2, 2, 2],
    ]);
    let actual = roots_of_unity(3, 6).unwrap();
    assert_eq!(expected, actual);
    assert!(actual.roots().try_len() == Ok(2));
    assert!(actual.roots().contains(&zadic_approx![3, 6, [1, 0, 0, 0, 0, 0]]));
    assert!(actual.roots().contains(&zadic_approx![3, 6, [2, 2, 2, 2, 2, 2]]));

    let expected = zadic_variety!(5, 6, [
        [1, 0, 0, 0, 0, 0],
        [2, 1, 2, 1, 3, 4],
        [3, 3, 2, 3, 1, 0],
        [4, 4, 4, 4, 4, 4],
    ]);
    let actual = roots_of_unity(5, 6).unwrap();
    assert_eq!(expected, actual);
    assert!(actual.roots().try_len() == Ok(4));
    assert!(actual.roots().contains(&zadic_approx![5, 6, [1, 0, 0, 0, 0, 0]]));
    assert!(actual.roots().contains(&zadic_approx![5, 6, [4, 4, 4, 4, 4, 4]]));

    let expected = zadic_variety!(7, 6, [
        [1, 0, 0, 0, 0, 0],
        [2, 4, 6, 3, 0, 2],
        [3, 4, 6, 3, 0, 2],
        [4, 2, 0, 3, 6, 4],
        [5, 2, 0, 3, 6, 4],
        [6, 6, 6, 6, 6, 6],
    ]);
    let actual = roots_of_unity(7, 6).unwrap();
    assert_eq!(expected, actual);
    assert!(actual.roots().try_len() == Ok(6));
    assert!(actual.roots().contains(&zadic_approx![7, 6, [1, 0, 0, 0, 0, 0]]));
    assert!(actual.roots().contains(&zadic_approx![7, 6, [6, 6, 6, 6, 6, 6]]));

}


// SLOW TESTS; enable when you want to fully test

#[ignore = "slow"]
#[test]
fn pth_root_slow() {

    // twentyfifthrt(32042220212._5) has a single root, 2
    println!("twenty_fifth rt of (32042220212._5) is 2._5...");
    let var2 = zadic_variety!(5, 8, [[2]]);
    assert_eq!(
        var2,
        nth_root(&iadic_pos!(5, [2, 1, 2, 0, 2, 2, 2, 4, 0, 2, 3]), 25, 8).unwrap()
    );

    // twentyfifthrt(231310332124302430341011314243033243440243222010001._5) has a single root, 101.5
    println!("twenty_fifth rt of (101.5^25) is 101._5...");
    let u26 = iadic_pos!(5, [1, 0, 1]);
    let var26 = ZAdicVariety::new(5, vec![u26.approximation(6)]);
    let u26_pow25 = iadic_pos!(5, [
        1, 0, 0, 0, 1, 0, 2, 2, 2, 3,
        4, 2, 0, 4, 4, 3, 4, 2, 3, 3,
        0, 3, 4, 2, 4, 1, 3, 1, 1, 0,
        1, 4, 3, 0, 3, 4, 2, 0, 3, 4,
        2, 1, 2, 3, 3, 0, 1, 3, 1, 3,
        2
    ]);
    assert_eq!(u26_pow25.signed_bigint_value(), u26.signed_bigint_value().pow(25u32));
    assert_eq!(var26, nth_root(&u26_pow25, 25, 6).unwrap());

    // nth-rt((-9/2)^n)
    println!("nth rt of (-9/2^n)...");
    let neg_nine_half = radic!(3, [0, 0], [1]);
    let sixthed = neg_nine_half.clone().pow(6);
    let zneg_nine_half = neg_nine_half.approximation(4);
    let zpos_nine_half = (-neg_nine_half.clone()).approximation(4);
    let var_both = ZAdicVariety::new(3, vec![zneg_nine_half.clone(), zpos_nine_half.clone()]);
    assert_eq!(var_both, nth_root(&sixthed, 6, 4).unwrap());

}

#[ignore = "very slow"]
#[test]
fn nth_root_many() {

    // Test nth_root over many integers and rationals

    // Test 5-adic positive integers
    let p = 5;
    let num_digits = 2;
    let pows = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 20, 25, 26];
    let roots = repeat_n(0..p, num_digits).multi_cartesian_product().map(
        |digits| UAdic::new(p, digits.into_iter().rev().collect())
    );
    for root in roots {
        let root_val = root.bigint_value();
        for power in pows {
            let root_powed = root.pow(power);
            let root_powed_val = root_powed.bigint_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = nth_root(&root_powed, power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root));
        }
    }

    // Test 3-adic rationals
    let p = 3;
    let fix_num = 2;
    let rep_num = 1;
    let pows = [1, 2, 3, 4, 5, 6, 7];
    let roots = repeat_n(0..p, fix_num).multi_cartesian_product().cartesian_product(
        repeat_n(0..p, rep_num).multi_cartesian_product()
    ).map(
        |(fixed_digits, repeat_digits)| RAdic::new(p, fixed_digits, repeat_digits)
    );
    for root in roots {
        let root_val = root.big_rational_value();
        for power in pows {
            let root_powed = root.pow(power);
            let root_powed_val = root_powed.big_rational_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = nth_root(&root_powed, power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root.truncation(6)));
        }
    }

}

#[ignore = "slow"]
#[test]
fn two_adic_many() {

    // Test nth_root over many 2-adic integers and rationals
    let p = 2;

    // Test 2-adic positive integers
    let num_digits = 3;
    let pows = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let roots = repeat_n(0..p, num_digits).multi_cartesian_product().map(
        |digits| UAdic::new(p, digits.into_iter().rev().collect())
    );
    for root in roots {
        let root_val = root.bigint_value();
        for power in pows {
            let root_powed = root.pow(power);
            let root_powed_val = root_powed.bigint_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = nth_root(&root_powed, power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root));
        }
    }

    // Test 2-adic rationals
    let fix_num = 2;
    let rep_num = 1;
    let pows = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let roots = repeat_n(0..p, fix_num).multi_cartesian_product().cartesian_product(
        repeat_n(0..p, rep_num).multi_cartesian_product()
    ).map(
        |(fixed_digits, repeat_digits)| RAdic::new(p, fixed_digits, repeat_digits)
    );
    for root in roots {
        let root_val = root.big_rational_value();
        for power in pows {
            let root_powed = root.pow(power);
            let root_powed_val = root_powed.big_rational_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = nth_root(&root_powed, power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root.truncation(6)));
        }
    }

    // Test 2-adic rationals (more digits, fewer powers)
    let fix_num = 2;
    let rep_num = 2;
    let pows = [1, 2, 3];
    let roots = repeat_n(0..p, fix_num).multi_cartesian_product().cartesian_product(
        repeat_n(0..p, rep_num).multi_cartesian_product()
    ).map(
        |(fixed_digits, repeat_digits)| RAdic::new(p, fixed_digits, repeat_digits)
    );
    for root in roots {
        let root_val = root.big_rational_value();
        for power in pows {
            let root_powed = root.pow(power);
            let root_powed_val = root_powed.big_rational_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = nth_root(&root_powed, power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root.truncation(6)));
        }
    }

}
