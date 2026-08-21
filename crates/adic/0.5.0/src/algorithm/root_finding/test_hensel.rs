use assertables::assert_matches;
use std::iter::repeat_n;
use itertools::Itertools;
use num::{traits::{Inv, Pow}, Rational32};
use crate::{
    error::AdicError,
    local_num::LocalZero,
    traits::{AdicInteger, AdicPrimitive, CanApproximate, PrimedFrom},
    EAdic, Polynomial, QAdic, RAdic, UAdic, Variety, ZAdic,
};

use crate::num_adic::test_util::e;


type EAdicPolynomial = Polynomial<EAdic>;


#[test]
fn sqrt_2() {

    // 2-adic
    let expected = Variety::empty();
    let actual = e::two2().nth_root(2, 3);
    assert_eq!(expected, actual.unwrap());
    let actual= EAdicPolynomial::new_with_prime(2, vec![-2, 0, 1]).variety(3);
    assert_eq!(expected.into_fractional(), actual.unwrap());

    // 3-adic
    let expected = Variety::empty();
    let actual= e::two3().nth_root(2, 3);
    assert_eq!(expected, actual.unwrap());
    let actual= EAdicPolynomial::new_with_prime(3, vec![-2, 0, 1]).variety(3);
    assert_eq!(expected.into_fractional(), actual.unwrap());

    // 5-adic
    let expected = Variety::empty();
    let actual= e::two().nth_root(2, 3);
    assert_eq!(expected, actual.unwrap());
    let actual= EAdicPolynomial::new_with_prime(5, vec![-2, 0, 1]).variety(3);
    assert_eq!(expected.into_fractional(), actual.unwrap());

    // 7-adic
    let expected = Variety::new(vec![
        zadic_approx!(7, 3, [3, 1, 2]),
        zadic_approx!(7, 3, [4, 5, 4]),
    ]);
    let actual = e::two7().nth_root(2, 3);
    assert_eq!(expected, actual.unwrap());

    let one_half = zadic_approx!(7, 3, [4, 3, 3]);
    let inverted_actual = one_half.nth_root(2, 3);
    let double_inverted_actual = Variety::new(
        inverted_actual.unwrap().into_roots().map(|rt| rt.inv()).collect()
    );
    assert_eq!(expected, double_inverted_actual);

    for zadic in expected.roots() {
        assert_eq!(zadic_approx!(7, 3, [2]), zadic.clone() * zadic.clone());
    }

    let expected = expected.into_fractional();
    let actual = EAdicPolynomial::new_with_prime(7, vec![-2, 0, 1]).variety(3);
    assert_eq!(expected, actual.unwrap());

    let expected = Variety::new(vec![
        QAdic::<ZAdic>::zero(7).into_approximation(6),
        QAdic::from_integer(zadic_approx!(7, 6, [0, 3, 1, 2, 6, 1])),
        QAdic::from_integer(zadic_approx!(7, 6, [0, 4, 5, 4, 0, 5])),
    ]);
    let actual = EAdicPolynomial::new_with_prime(7, vec![0, 98, 0, -1]).variety(6);
    assert_eq!(expected, actual.unwrap());

}

#[test]
fn nth_root_certainty() {

    let p = 7;
    let n = 2;

    // Cases:
    // square root of 2 in the error case
    let certainty = 1;
    let precision = 2;

    let actual = zadic_approx!(p, certainty, [2]).nth_root(n, precision);
    assert_matches!(actual, Err(AdicError::InappropriatePrecision(_)));

    // sqrt 2 in the ok case
    let certainty = 2;
    let precision = 2;

    let actual = zadic_approx!(p, certainty, [2]).nth_root(n, precision);
    assert_matches!(actual, Ok(_));

    // sqrt 98 in the error
    let certainty = 3;
    let precision = 3;

    let actual = zadic_approx!(p, certainty, [0, 0, 2]).nth_root(n, precision);
    assert_matches!(actual, Err(AdicError::InappropriatePrecision(_)));

    // sqrt 98 in the ok
    let certainty = 3;
    let precision = 2;

    let actual = zadic_approx!(p, certainty, [0, 0, 2]).nth_root(n, precision);
    assert_matches!(actual, Ok(_));

}

#[test]
fn simple_poly() {

    let precision = 6;

    for p in [2, 3, 5, 7, 11, 13] {

        // (x - 1)
        let f = EAdicPolynomial::new_with_prime(p, vec![-1, 1]);
        assert_eq!(1, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(p, vec![1]).into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // (x + 1)
        let f = EAdicPolynomial::new_with_prime(p, vec![1, 1]);
        assert_eq!(1, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(p, vec![-1]).into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // (4x - 5)
        let f = EAdicPolynomial::new_with_prime(p, vec![-5, 4]);
        assert_eq!(1, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::try_primed_from_roots(
            p, vec![Rational32::new(5, 4)]
        ).unwrap().into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // x^2 - 3x + 2
        // (x - 1)(x - 2)
        let f = EAdicPolynomial::new_with_prime(p, vec![2, -3, 1]);
        assert_eq!(2, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(p, vec![1, 2]).into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // x^2 + 3x + 2
        // (x + 1)(x + 2)
        let f = EAdicPolynomial::new_with_prime(p, vec![2, 3, 1]);
        assert_eq!(2, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(p, vec![-1, -2]).into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // x^2 - 6x - 7
        // (x - 7)(x + 1)
        let f = EAdicPolynomial::new_with_prime(p, vec![-7, -6, 1]);
        assert_eq!(2, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(p, vec![7, -1]).into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // x^2 - 1
        // (x - 1)(x + 1)
        let f = EAdicPolynomial::new_with_prime(p, vec![-1, 0, 1]);
        assert_eq!(2, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(p, vec![1, -1]).into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // x^2 - x
        // (x - 1)(x)
        let f = EAdicPolynomial::new_with_prime(p, vec![0, -1, 1]);
        assert_eq!(2, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(p, vec![1, 0]).into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // x^3 + 4x^2 - 4x - 16
        // (x - 2)(x + 2)(x + 4)
        let f = EAdicPolynomial::new_with_prime(p, vec![-16, -4, 4, 1]);
        assert_eq!(3, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(p, vec![2, -2, -4]).into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

    }

}

#[test]
fn general_poly() {

    let precision = 6;
    for p in [2, 3, 5, 7, 11, 13] {

        // Fractional root
        // (2x - 1)(x + 2)
        // 2x^2 + 3x - 2
        let f = EAdicPolynomial::new_with_prime(p, vec![-2, 3, 2]);
        assert_eq!(2, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::try_primed_from_roots(
            p, vec![Rational32::new(1, 2), Rational32::new(-2, 1)]
        ).unwrap().into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // (x)(x - 1)(x - 2)(x - 3)(x - 4)
        // x^5 - 10x^4 + 35x^3 - 50x^2 + 24x
        let f = EAdicPolynomial::new_with_prime(p, vec![0, 24, -50, 35, -10, 1]);
        assert_eq!(5, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(
            p, vec![0, 1, 2, 3, 4]
        ).into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

        // (3x + 2)(4x - 5)
        // 12x^2 - 7x - 10
        let f = EAdicPolynomial::new_with_prime(p, vec![-10, -7, 12]);
        assert_eq!(2, f.variety_size().unwrap());
        let expected = Variety::<QAdic<ZAdic>>::try_primed_from_roots(
            p, vec![Rational32::new(-2, 3), Rational32::new(5, 4)]
        ).unwrap().into_approximation(precision);
        let actual = f.variety(precision).unwrap();
        assert_eq!(expected, actual);

    }

    // Irrational number
    // x (x^2 - 2)
    let p = 7;
    let f = EAdicPolynomial::new_with_prime(p, vec![0, -2, 0, 1]);
    assert_eq!(3, f.variety_size().unwrap());
    let expected = Variety::new(vec![
        QAdic::<ZAdic>::zero(7).into_approximation(6),
        QAdic::from_integer(zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2])),
        QAdic::from_integer(zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4])),
    ]);
    let actual = f.variety(precision).unwrap();
    assert_eq!(expected, actual);

    // Degree = p
    // (x - 1)(x)(x + 1)
    // x^3 - x
    let p = 3;
    let f = EAdicPolynomial::new_with_prime(p, vec![0, -1, 0, 1]);
    assert_eq!(3, f.variety_size().unwrap());
    let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(
        p, vec![1, 0, -1]
    ).into_approximation(precision);
    let actual = f.variety(precision).unwrap();
    assert_eq!(expected, actual);
    let t = ZAdic::teichmuller(p, precision.try_into().unwrap()).unwrap();
    assert_eq!(expected.try_into_integer().unwrap(), t);

    // ...001._5x^2 + ...343._5x^1 + ...101._5x^0
    let f = EAdicPolynomial::new_with_prime(p, vec![26, -27, 1]);
    let precision = 3;
    assert_eq!(2, f.variety_size().unwrap());
    let expected = Variety::<QAdic<ZAdic>>::primed_from_roots(
        p, vec![1, 26]
    ).into_approximation(precision);
    let actual = f.variety(precision).unwrap();
    assert_eq!(expected, actual);

    assert_eq!(Ok(0), UAdic::new(7, vec![2]).num_nth_roots(0));
    assert_eq!(Ok(1), UAdic::new(7, vec![2]).num_nth_roots(1));
    assert_eq!(Ok(2), UAdic::new(7, vec![2]).num_nth_roots(2));
    assert_eq!(Ok(0), UAdic::new(7, vec![2]).num_nth_roots(3));
    assert_eq!(Ok(2), UAdic::new(7, vec![2]).num_nth_roots(4));
    assert_eq!(Ok(1), UAdic::new(7, vec![2]).num_nth_roots(5));
    assert_eq!(Ok(0), UAdic::new(7, vec![2]).num_nth_roots(6));
    assert_eq!(Ok(0), UAdic::new(7, vec![2]).num_nth_roots(7));

}

#[test]
fn fractional_roots() {

    // Root is not an adic integer
    let p = 5;

    // 5-adic (5x - 1)
    let f = EAdicPolynomial::new_with_prime(p, vec![-1, 5]);
    let expected = Variety::new(vec![
        QAdic::new(zadic_approx!(5, 7, [1]), -1),
    ]);
    let actual = f.variety(6).unwrap();
    assert_eq!(expected, actual);

    // 5-adic (5x - 1)(x + 2)
    // 5x^2 + 9x - 2
    let f = EAdicPolynomial::new_with_prime(p, vec![-2, 9, 5]);
    let expected = Variety::new(vec![
        QAdic::new(zadic_approx!(5, 7, [1]), -1),
        QAdic::from_integer(zadic_approx!(5, 6, [3, 4, 4, 4, 4, 4])),
    ]);
    let actual = f.variety(6).unwrap();
    assert_eq!(expected, actual);

    // 7-adic (x + 1/7)(x^2 - 2/49)(x^2 - 3)
    // 6/343 + 6/49 x - 149/343 x^2 - 149/49 x^3 + 1/7 x^4 + x^5
    let p = 7;
    let f = Polynomial::<QAdic<EAdic>>::new_with_prime(p, vec![
        Rational32::new(6, 343), Rational32::new(6, 49), Rational32::new(-149, 343),
        Rational32::new(-149, 49), Rational32::new(1, 7), Rational32::new(1, 1)
    ]);
    assert_eq!(Ok(3), f.variety_size());
    let expected = Variety::new(vec![
        QAdic::new(zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]), -1),
        QAdic::new(zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]), -1),
        QAdic::new(ZAdic::primed_from(p, -1).into_approximation(6), -1),
    ]);
    let actual = f.variety(5).unwrap();
    assert_eq!(expected, actual);

}

#[test]
fn degenerate_roots() {

    // (x - 1)^2 = x^2 - 2 x + 1
    let p = 5;
    let precision = 3;
    let f = EAdicPolynomial::new_with_prime(p, vec![1, -2, 1]);
    assert_eq!(2, f.variety_size().unwrap());
    let expected = Variety::new(vec![
        QAdic::from_integer(zadic_approx!(p, precision, [1])),
        QAdic::from_integer(zadic_approx!(p, precision, [1])),
    ]);
    let actual = f.variety(precision as isize).unwrap();
    assert_eq!(expected, actual);

    // (5x - 1)^2 = 25 x^2 - 10 x + 1
    let p = 5;
    let precision = 3;
    let f = EAdicPolynomial::new_with_prime(p, vec![1, -10, 25]);
    assert_eq!(2, f.variety_size().unwrap());
    let expected = Variety::new(vec![
        QAdic::new(zadic_approx!(p, precision+1, [1]), -1),
        QAdic::new(zadic_approx!(p, precision+1, [1]), -1),
    ]);
    let actual = f.variety(precision as isize).unwrap();
    assert_eq!(expected, actual);

    // (x - 26)^2 = x^2 - 52 x + 676
    let p = 5;
    let precision = 3;
    let f = EAdicPolynomial::new_with_prime(p, vec![676, -52, 1]);
    assert_eq!(2, f.variety_size().unwrap());
    let expected = Variety::new(vec![
        QAdic::from_integer(zadic_approx!(p, precision, [1, 0, 1])),
        QAdic::from_integer(zadic_approx!(p, precision, [1, 0, 1])),
    ]);
    let actual = f.variety(precision as isize).unwrap();
    assert_eq!(expected, actual);

    // (x - 1)^2 (x - 26) = x^3 - 28 x^2 + 53 x - 26
    let p = 5;
    let precision = 4;
    let f = EAdicPolynomial::new_with_prime(p, vec![-26, 53, -28, 1]);
    assert_eq!(3, f.variety_size().unwrap());
    let expected = Variety::new(vec![
        QAdic::from_integer(zadic_approx!(p, precision, [1])),
        QAdic::from_integer(zadic_approx!(p, precision, [1])),
        QAdic::from_integer(zadic_approx!(p, precision, [1, 0, 1])),
    ]);
    let actual = f.variety(precision as isize).unwrap();
    assert_eq!(expected, actual);

    let precision = 5;
    let f = EAdicPolynomial::new_with_prime(p, vec![-26, 53, -28, 1]);
    assert_eq!(3, f.variety_size().unwrap());
    let expected = Variety::new(vec![
        QAdic::from_integer(zadic_approx!(p, precision, [1])),
        QAdic::from_integer(zadic_approx!(p, precision, [1])),
        QAdic::from_integer(zadic_approx!(p, precision, [1, 0, 1])),
    ]);
    let actual = f.variety(precision as isize).unwrap();
    assert_eq!(expected, actual);

    // (x - 1)^2 (x - 26)^2 = x^4 - 54 x^3 + 781 x^2 - 1404 x + 676
    let p = 5;
    let precision = 3;
    let f = EAdicPolynomial::new_with_prime(p, vec![676, -1404, 781, -54, 1]);
    assert_eq!(4, f.variety_size().unwrap());
    let expected = Variety::new(vec![
        QAdic::from_integer(zadic_approx!(p, precision, [1])),
        QAdic::from_integer(zadic_approx!(p, precision, [1])),
        QAdic::from_integer(zadic_approx!(p, precision, [1, 0, 1])),
        QAdic::from_integer(zadic_approx!(p, precision, [1, 0, 1])),
    ]);
    let actual = f.variety(precision as isize).unwrap();
    assert_eq!(expected, actual);

    // (5x - 1)^2 (5x - 26)^2 = 625x^4 - 6750 x^3 + 19525 x^2 - 7020 x + 676
    let p = 5;
    let precision = 3;
    let f = EAdicPolynomial::new_with_prime(p, vec![676, -7020, 19525, -6750, 625]);
    assert_eq!(4, f.variety_size().unwrap());
    let expected = Variety::new(vec![
        QAdic::new(zadic_approx!(p, precision+1, [1]), -1),
        QAdic::new(zadic_approx!(p, precision+1, [1]), -1),
        QAdic::new(zadic_approx!(p, precision+1, [1, 0, 1]), -1),
        QAdic::new(zadic_approx!(p, precision+1, [1, 0, 1]), -1),
    ]);
    let actual = f.variety(precision as isize).unwrap();
    assert_eq!(expected, actual);

}

#[test]
fn precision_lower_than_guess() {

    // f(x) = (x - 1) (x - ...1111) = x^2 - ...1112 x + ...1111 = x^2 + ...3333 x + ...1111
    let p = 5;
    let f = Polynomial::<ZAdic>::new(vec![
        zadic_approx!(p, 4, [1, 1, 1, 1]),
        zadic_approx!(p, 4, [3, 3, 3, 3]),
        ZAdic::one(p),
    ]);
    let expected = Variety::new(vec![
        QAdic::from_integer(zadic_approx!(p, 1, [1])),
        QAdic::from_integer(zadic_approx!(p, 1, [1])),
    ]);
    let actual = f.variety(1).unwrap();
    assert_eq!(expected, actual);

    let expected = Variety::new(vec![
        QAdic::from_integer(zadic_approx!(p, 2, [1, 0])),
        QAdic::from_integer(zadic_approx!(p, 2, [1, 1])),
    ]);
    let actual = f.variety(2).unwrap();
    assert_eq!(expected, actual);

}

#[test]
fn a_equals_zero() {

    // Returns multiple solutions

    let precision = 3;
    let iprecision = isize::try_from(precision).unwrap();
    let zero = ZAdic::zero(5).into_approximation(precision);
    let expected_variety = Variety::new(vec![zero.clone(), zero.clone()]);
    let expected_size = 2;

    let actual = e::zero().nth_root(2, precision);
    assert_eq!(expected_variety, actual.unwrap());
    assert_eq!(expected_size, e::zero().num_nth_roots(2).unwrap());

    let actual = EAdicPolynomial::new_with_prime(5, vec![0, 0, 1]).variety(iprecision);
    assert_eq!(expected_variety.into_fractional(), actual.unwrap());
    assert_eq!(expected_size, EAdicPolynomial::new_with_prime(5, vec![0, 0, 1]).variety_size().unwrap());

}

#[test]
fn const_polynomial() {

    let precision = 3;

    let expected = Ok(Variety::empty());
    let actual = EAdicPolynomial::new_with_prime(5, vec![1]).variety(precision);
    assert_eq!(expected, actual);

    let expected = Ok(Variety::empty());
    let actual = EAdicPolynomial::new(vec![]).variety(precision);
    assert_eq!(expected, actual);

}

#[test]
fn nth_root_of_p() {

    // 5-adic

    // sqrt(5^m)
    let expected = Variety::empty();
    let actual = e::five().nth_root(2, 4);
    assert_eq!(0, e::five().num_nth_roots(2).unwrap());
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(5, 4, [
        [0, 1, 0, 0],
        [0, 4, 4, 4],
    ]);
    let actual = e::twenty_five().nth_root(2, 4);
    assert_eq!(2, e::twenty_five().num_nth_roots(2).unwrap());
    assert_eq!(expected, actual.unwrap());
    let expected = Variety::empty();
    let actual = e::one_twenty_five().nth_root(2, 4);
    assert_eq!(0, e::one_twenty_five().num_nth_roots(2).unwrap());
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(5, 4, [
        [0, 0, 1, 0],
        [0, 0, 4, 4],
    ]);
    let actual = e::six_twenty_five().nth_root(2, 4);
    assert_eq!(2, e::six_twenty_five().num_nth_roots(2).unwrap());
    assert_eq!(expected, actual.unwrap());

    // cubert(5^m)
    let expected = Variety::empty();
    let actual = e::five().nth_root(3, 4);
    assert_eq!(0, e::five().num_nth_roots(3).unwrap());
    assert_eq!(expected, actual.unwrap());
    let expected = Variety::empty();
    let actual = e::twenty_five().nth_root(3, 4);
    assert_eq!(0, e::twenty_five().num_nth_roots(3).unwrap());
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(5, 4, [
        [0, 1, 0],
    ]);
    let actual = e::one_twenty_five().nth_root(3, 4);
    assert_eq!(1, e::one_twenty_five().num_nth_roots(3).unwrap());
    assert_eq!(expected, actual.unwrap());
    let expected = Variety::empty();
    let actual = e::six_twenty_five().nth_root(3, 4);
    assert_eq!(0, e::six_twenty_five().num_nth_roots(3).unwrap());
    assert_eq!(expected, actual.unwrap());

    // fourthrt(5^m)
    let expected = Variety::empty();
    let actual = e::five().nth_root(4, 4);
    assert_eq!(0, e::five().num_nth_roots(4).unwrap());
    assert_eq!(expected, actual.unwrap());
    let expected = Variety::empty();
    let actual = e::twenty_five().nth_root(4, 4);
    assert_eq!(0, e::twenty_five().num_nth_roots(4).unwrap());
    assert_eq!(expected, actual.unwrap());
    let expected = Variety::empty();
    let actual = e::one_twenty_five().nth_root(4, 4);
    assert_eq!(0, e::one_twenty_five().num_nth_roots(4).unwrap());
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(5, 4, [
        [0, 1, 0, 0],
        [0, 2, 1, 2],
        [0, 3, 3, 2],
        [0, 4, 4, 4],
    ]);
    let actual = e::six_twenty_five().nth_root(4, 4);
    assert_eq!(4, e::six_twenty_five().num_nth_roots(4).unwrap());
    assert_eq!(expected, actual.unwrap());

    // 7-adic

    let expected = zadic_variety!(7, 5, [
        [0, 3, 1, 2, 6],
        [0, 4, 5, 4, 0],
    ]);
    let actual = e::ninety_eight7().nth_root(2, 5);
    assert_eq!(2, e::ninety_eight7().num_nth_roots(2).unwrap());
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
    let expected = Variety::new(vec![solution.clone()]);
    let actual = e::ten3().nth_root(3, 8);
    assert_eq!(expected, actual.unwrap());
    assert_eq!(ZAdic::primed_from(3, 10).approximation(8), solution.pow(3));

    // fifthrt(1._5) has a single root, 1
    assert_eq!(
        zadic_variety!(5, 8, [[1]]),
        UAdic::one(5).nth_root(5, 8).unwrap()
    );

    // fifthrt(112._5) has a single root, 2
    let var2 = zadic_variety!(5, 8, [[2]]);
    assert_eq!(var2, eadic!(5, [2, 1, 1]).nth_root(5, 8).unwrap());

    // fifthrt((12._5)^5) has one root: 12._5
    let seven = eadic!(5, [2, 1]);
    let seven_to_fifth = eadic!(5, [2, 1, 2, 4, 1, 0, 1]);
    assert_eq!(seven.clone().pow(5), seven_to_fifth);
    let expected = Variety::new(vec![
        seven.into_approximation(4)
    ]);
    let actual = seven_to_fifth.into_approximation(20).nth_root(5, 4);
    assert_eq!(expected, actual.unwrap());

    // nth-rt((-9/2)^n)
    let neg_nine_half = eadic_rep!(3, [0, 0], [1]);
    assert_eq!(Ok(Rational32::new(-9, 2)), neg_nine_half.rational_value());
    let squared = neg_nine_half.clone().pow(2);
    assert_eq!(Ok(Rational32::new(81, 4)), squared.rational_value());
    let cubed = neg_nine_half.clone().pow(3);
    assert_eq!(Ok(Rational32::new(-729, 8)), cubed.rational_value());
    let zneg_nine_half = neg_nine_half.approximation(4);
    let zpos_nine_half = (-neg_nine_half.clone()).approximation(4);
    let var_one = Variety::new(vec![zneg_nine_half.clone()]);
    let var_both = Variety::new(vec![zneg_nine_half.clone(), zpos_nine_half.clone()]);
    assert_eq!(var_both, squared.nth_root(2, 4).unwrap());
    assert_eq!(var_one, cubed.nth_root(3, 4).unwrap());
    let fourthed = neg_nine_half.clone().pow(4);
    assert_eq!(var_both, fourthed.nth_root(4, 4).unwrap());
    let fifthed = neg_nine_half.clone().pow(5);
    assert_eq!(var_one, fifthed.nth_root(5, 4).unwrap());

    // twentyfifthrt(32042220212._5) has a single root, 2
    let var2 = zadic_variety!(5, 8, [[2]]);
    assert_eq!(
        var2,
        eadic!(5, [2, 1, 2, 0, 2, 2, 2, 4, 0, 2, 3]).nth_root(25, 8).unwrap()
    );

    // twentyfifthrt(231310332124302430341011314243033243440243222010001._5) has a single root, 101.5
    let u26 = eadic!(5, [1, 0, 1]);
    let var26 = Variety::new(vec![u26.approximation(6)]);
    let u26_pow25 = eadic!(5, [
        1, 0, 0, 0, 1, 0, 2, 2, 2, 3,
        4, 2, 0, 4, 4, 3, 4, 2, 3, 3,
        0, 3, 4, 2, 4, 1, 3, 1, 1, 0,
        1, 4, 3, 0, 3, 4, 2, 0, 3, 4,
        2, 1, 2, 3, 3, 0, 1, 3, 1, 3,
        2
    ]);
    assert_eq!(u26_pow25.signed_bigint_value().unwrap(), u26.signed_bigint_value().unwrap().pow(25u32));
    assert_eq!(var26, u26_pow25.nth_root(25, 6).unwrap());

}

#[test]
fn two_adic() {

    // sqrt(2^m)
    let expected = Variety::empty();
    let actual = e::two2().nth_root(2, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(2, 4, [
        [0, 1, 0, 0],
        [0, 1, 1, 1],
    ]);
    let actual = e::four2().nth_root(2, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = Variety::empty();
    let actual = e::eight2().nth_root(2, 4);
    assert_eq!(expected, actual.unwrap());
    let expected = zadic_variety!(2, 4, [
        [0, 0, 1, 0],
        [0, 0, 1, 1],
    ]);
    let actual = e::sixteen2().nth_root(2, 4);
    assert_eq!(expected, actual.unwrap());

    // sqrt(3)
    let expected = Variety::empty();
    let actual = e::three2().nth_root(2, 4);
    assert_eq!(expected, actual.unwrap());

    // sqrt(17)
    let expected = zadic_variety!(2, 8, [
        [1, 0, 0, 1, 0, 1, 1, 1],
        [1, 1, 1, 0, 1, 0, 0, 0],
    ]);
    let actual = e::seventeen2().nth_root(2, 8);
    assert_eq!(expected, actual.unwrap());

    let expected = zadic_variety!(2, 8, [
        [1, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1, 1, 1],
    ]);
    let actual = e::one2().nth_root(2, 8).unwrap();
    assert_eq!(expected, actual);
    let actual = e::one2().nth_root(4, 8).unwrap();
    assert_eq!(expected, actual);
    let actual = e::one2().nth_root(6, 8).unwrap();
    assert_eq!(expected, actual);
    let actual = e::one2().nth_root(8, 8).unwrap();
    assert_eq!(expected, actual);
    let actual = e::one2().nth_root(10, 8).unwrap();
    assert_eq!(expected, actual);

    let expected = zadic_variety!(2, 8, [
        [1, 0, 1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 0, 0, 0],
    ]);
    let eighty_one = eadic!(2, [1, 0, 0, 0, 1, 0, 1]);
    let actual = eighty_one.nth_root(4, 8).unwrap();
    assert_eq!(expected, actual);

}

#[test]
fn sqrt_fractions() {

    // 2-adic

    let pos_one_ninth = eadic_rep!(2, [1], [0, 0, 1, 1, 1, 0]);
    let neg_one_third = eadic_rep!(2, [], [1, 0]);
    let pos_one_third = eadic_rep!(2, [1], [1, 0]);
    let expected = Variety::new(vec![
        neg_one_third.into_approximation(6),
        pos_one_third.into_approximation(6),
    ]);
    let actual = pos_one_ninth.nth_root(2, 6).unwrap();
    assert_eq!(expected, actual);

    let one_fourth = QAdic::<EAdic>::primed_from(2, Rational32::new(1, 4));
    let expected = Variety::<QAdic<EAdic>>::primed_from_roots(
        2,
        vec![Rational32::new(1, 2), Rational32::new(-1, 2)]
    ).into_approximation(6);
    let actual = one_fourth.nth_root(2, 6).unwrap();
    assert_eq!(expected, actual);

    // 5-adic

    let expected = Variety::new(vec![
        e::neg_1_2().into_approximation(6),
        e::pos_1_2().into_approximation(6),
    ]);
    let actual = e::pos_1_4().nth_root(2, 6).unwrap();
    assert_eq!(expected, actual);

    // 7-adic

    let pos_one_sixteenth = eadic_rep!(7, [4], [6, 3]);
    let neg_one_fourth = eadic_rep!(7, [], [5, 1]);
    let pos_one_fourth = eadic_rep!(7, [2], [5, 1]);
    let neg_one_half = eadic_rep!(7, [], [3]);
    let pos_one_half = eadic_rep!(7, [4], [3]);
    let expected = Variety::new(vec![
        pos_one_fourth.approximation(6),
        neg_one_fourth.approximation(6),
    ]);
    let actual = pos_one_sixteenth.nth_root(2, 6).unwrap();
    assert_eq!(expected, actual);
    let expected = Variety::new(vec![
        neg_one_half.into_approximation(6),
        pos_one_half.into_approximation(6),
    ]);
    let actual = pos_one_fourth.nth_root(2, 6).unwrap();
    assert_eq!(expected, actual);

    let two_fourty_ninths = QAdic::<EAdic>::primed_from(7, Rational32::new(2, 49));
    let expected = Variety::new(vec![
        QAdic::new(zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]), -1),
        QAdic::new(zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]), -1),
    ]);
    let actual = two_fourty_ninths.nth_root(2, 5).unwrap();
    assert_eq!(expected, actual);

}


// SLOW TESTS; enable when you want to fully test

#[ignore = "slow"]
#[test]
fn pth_root_slow() {

    // nth-rt((-9/2)^n)
    println!("nth rt of (-9/2^n)...");
    let neg_nine_half = eadic_rep!(3, [0, 0], [1]);
    let sixthed = neg_nine_half.clone().pow(6);
    println!("sixthed: {sixthed}");
    let zneg_nine_half = neg_nine_half.approximation(4);
    let zpos_nine_half = (-neg_nine_half.clone()).approximation(4);
    let var_both = Variety::new(vec![zneg_nine_half.clone(), zpos_nine_half.clone()]);
    assert_eq!(var_both, sixthed.nth_root(6, 4).unwrap());

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
            let root_powed = root.clone().pow(power);
            let root_powed_val = root_powed.bigint_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = root_powed.nth_root(power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            assert!(variety.roots().any(|var_root| *var_root == root.approximation(6)));
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
            let root_powed = root.clone().pow(power);
            let root_powed_val = root_powed.big_rational_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = root_powed.nth_root(power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            assert!(variety.roots().any(|var_root| *var_root == root.approximation(6)));
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
            let root_powed = root.clone().pow(power);
            let root_powed_val = root_powed.bigint_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = root_powed.nth_root(power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            assert!(variety.roots().any(|var_root| *var_root == root.approximation(6)));
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
            let root_powed = root.clone().pow(power);
            let root_powed_val = root_powed.big_rational_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = root_powed.nth_root(power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            if root.is_local_zero() {
                assert!(variety.roots().any(ZAdic::is_local_zero));
            } else {
                assert!(variety.roots().any(|var_root| *var_root == root.approximation(6)));
            }
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
            let root_powed = root.clone().pow(power);
            let root_powed_val = root_powed.big_rational_value();
            assert_eq!(root_val.clone().pow(power), root_powed_val);
            println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
            let variety = root_powed.nth_root(power, 6).unwrap();
            println!("[{}]", variety.roots().map(ToString::to_string).join(", "));
            if root.is_local_zero() {
                assert!(variety.roots().any(ZAdic::is_local_zero));
            } else {
                assert!(variety.roots().any(|var_root| *var_root == root.approximation(6)));
            }
        }
    }

}
