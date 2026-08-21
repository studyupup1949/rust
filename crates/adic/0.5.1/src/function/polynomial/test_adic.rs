use num::{BigInt, Num, One, Rational32, Zero};
use crate::{
    mapping::{Differentiable, Mapping},
    normed::Valuation,
    traits::{AdicPrimitive, CanApproximate, PrimedFrom, PrimedInto, TryPrimedFrom},
    EAdic, QAdic, Variety, ZAdic,
};
use super::{euclid, zadic_wrapper, Polynomial, NewtonPolygon};


type EAdicPolynomial = Polynomial<EAdic>;


#[test]
fn new() {
    let _ = Polynomial::<EAdic>::new(vec![EAdic::one(5)]);
    let _ = Polynomial::<EAdic>::new_with_prime(5, vec![1]);
    let _ = Polynomial::<EAdic>::primed_from(5, Polynomial::new(vec![1]));
    let _: Polynomial<EAdic> = Polynomial::new(vec![1]).primed_into(5);
}

#[test]
#[should_panic(expected="MixedCharacteristic")]
fn mismatched_p() {
    let poly = EAdicPolynomial::new(vec![EAdic::zero(3), EAdic::one(3)]);
    let _ = poly.eval(EAdic::zero(5));
}

#[test]
fn degree() {
    assert_eq!(EAdicPolynomial::new_with_prime(5, vec![-2, 0, 1]).degree(), Some(2));
    assert_eq!(EAdicPolynomial::new_with_prime(5, vec![-1, 1]).degree(), Some(1));
    assert_eq!(EAdicPolynomial::new_with_prime(5, vec![1]).degree(), Some(0));
    assert_eq!(EAdicPolynomial::new(vec![]).degree(), None);
}

#[test]
fn monic() {

    // 3-adic `4x-5 -> x-5/4 = x + 2/(1-9) - 1 = x + (20)1._3`
    let poly = EAdicPolynomial::new_with_prime(3, vec![-5, 4]);
    let pmonic_expected_coeffs = vec![Rational32::new(-5, 4), Rational32::new(1, 1)]
        .into_iter()
        .map(|r| EAdic::try_primed_from(3, r).unwrap())
        .collect();
    assert_eq!(Polynomial::new(pmonic_expected_coeffs), poly.primitive_monic());

    // 3-adic `...00011x-5 -> x-(5/...00011) = x + ...20201._3
    let poly = Polynomial::new(vec![
        ZAdic::primed_from(3, -5),
        ZAdic::new_approx(3, 5, vec![1, 1, 0, 0, 0]),
    ]);
    let pmonic_expected_coeffs = vec![
        ZAdic::new_approx(3, 5, vec![1, 0, 2, 0, 2]),
        ZAdic::one(3),
    ];
    assert_eq!(Polynomial::new(pmonic_expected_coeffs), poly.primitive_monic());

    // 5-adic `-50x^2 + 15x - 25 -> 5x^2 - (3/2)x + 5(1/2)`
    let poly = EAdicPolynomial::new_with_prime(5, vec![-25, 15, -50]);
    let pmonic_expected_coeffs = vec![Rational32::new(5, 2), -Rational32::new(3, 2), Rational32::new(5, 1)]
        .into_iter()
        .map(|r| EAdic::try_primed_from(5, r).unwrap())
        .collect();
    assert_eq!(Polynomial::new(pmonic_expected_coeffs), poly.primitive_monic());

    // 5-adic `-50x^2 + ...00030x - ...00100 -> 5x^2 + ...2221x + ...2230`
    let poly = Polynomial::new(vec![
        ZAdic::new_approx(5, 5, vec![0, 0, 4, 4, 4]),
        ZAdic::new_approx(5, 5, vec![0, 3, 0, 0, 0]),
        ZAdic::primed_from(5, -50),
    ]);
    let pmonic_expected_coeffs = vec![
        ZAdic::new_approx(5, 4, vec![0, 3, 2, 2]),
        ZAdic::new_approx(5, 4, vec![1, 2, 2, 2]),
        ZAdic::primed_from(5, 5),
    ];
    assert_eq!(Polynomial::new(pmonic_expected_coeffs), poly.primitive_monic());

    // 2-adic `-120x^5 + 48x^2 - 28 -> 2x^5 - (4/5)x^2 + (7/15)`
    let poly = EAdicPolynomial::new_with_prime(2, vec![-28, 0, 48, 0, 0, -120]);
    let pmonic_expected_coeffs = vec![
        Rational32::new(7, 15), Rational32::zero(), -Rational32::new(4, 5),
        Rational32::zero(), Rational32::zero(), Rational32::new(2, 1)
    ].into_iter()
        .map(|r| EAdic::try_primed_from(2, r).unwrap())
        .collect();
    assert_eq!(Polynomial::new(pmonic_expected_coeffs), poly.primitive_monic());

    // 2-adic `-120x^5 + ...011000x^2 - 28 -> 2x^5 + ...110x^2 + (7/15)`
    let poly = Polynomial::new(vec![
        ZAdic::primed_from(2, -28),
        ZAdic::zero(2),
        ZAdic::new_approx(2, 6, vec![0, 0, 0, 1, 1]),
        ZAdic::zero(2),
        ZAdic::zero(2),
        ZAdic::primed_from(2, -120),
    ]);
    let pmonic_expected_coeffs = vec![
        ZAdic::try_primed_from(2, Rational32::new(7, 15)).unwrap(),
        ZAdic::zero(2),
        ZAdic::new_approx(2, 4, vec![0, 1, 1]),
        ZAdic::zero(2),
        ZAdic::zero(2),
        ZAdic::primed_from(2, 2),
    ];
    assert_eq!(Polynomial::new(pmonic_expected_coeffs), poly.primitive_monic());

    // 5-adic `7130 -> 1`
    assert_eq!(Polynomial::new_with_prime(5, vec![1]), Polynomial::<EAdic>::new_with_prime(5, vec![7130]).primitive_monic());

    // 5-adic `...12130 -> 1`
    assert_eq!(
        Polynomial::new_with_prime(5, vec![1]),
        Polynomial::new(vec![ZAdic::new_approx(5, 5, vec![0, 3, 1, 2, 1])]).primitive_monic()
    );

    // 5-adic `-50x^2 + 15x - 25 -> x^2 - (3/10)x + (1/2)`
    let poly = EAdicPolynomial::new_with_prime(5, vec![-25, 15, -50]);
    let monic_expected_coeffs = vec![Rational32::new(1, 2), -Rational32::new(3, 10), Rational32::new(1, 1)]
        .into_iter()
        .map(|r| QAdic::<EAdic>::primed_from(5, r))
        .collect();
    assert_eq!(Polynomial::new(monic_expected_coeffs), poly.monic());

    // 5-adic `...4300x^2 + 15x - 25 -> x^2 + ...2.1x + ...23`
    let poly = Polynomial::new(vec![
        ZAdic::primed_from(5, -25),
        ZAdic::primed_from(5, 15),
        ZAdic::new_approx(5, 4, vec![0, 0, 3, 4]),
    ]);
    let monic_expected_coeffs = vec![
        QAdic::new(ZAdic::new_approx(5, 2, vec![3, 2]), 0),
        QAdic::new(ZAdic::new_approx(5, 2, vec![1, 2]), -1),
        QAdic::one(5),
    ];
    assert_eq!(Polynomial::new(monic_expected_coeffs), poly.monic());

    // 2-adic `-120x^5 + 48x^2 - 28 -> x^5 - (2/5)x^2 + (7/30)`
    let poly = EAdicPolynomial::new_with_prime(2, vec![-28, 0, 48, 0, 0, -120]);
    let pmonic_expected_coeffs = vec![
        Rational32::new(7, 30), Rational32::zero(), -Rational32::new(2, 5),
        Rational32::zero(), Rational32::zero(), Rational32::one(),
    ].into_iter()
        .map(|r| QAdic::<EAdic>::primed_from(2, r))
        .collect();
    assert_eq!(Polynomial::new(pmonic_expected_coeffs), poly.monic());

    // 2-adic `...10001000x^5 + ...110000x^2 - 28 -> x^5  + ...110x^2 + ...01001`
    let poly = Polynomial::new(vec![
        ZAdic::primed_from(2, -28),
        ZAdic::zero(2),
        ZAdic::new_approx(2, 6, vec![0, 0, 0, 0, 1, 1]),
        ZAdic::zero(2),
        ZAdic::zero(2),
        ZAdic::new_approx(2, 8, vec![0, 0, 0, 1, 0, 0, 0, 1]),
    ]);
    let pmonic_expected_coeffs = vec![
        QAdic::new(ZAdic::new_approx(2, 5, vec![1, 0, 0, 1, 0]), -1),
        QAdic::zero(2),
        QAdic::new(ZAdic::new_approx(2, 3, vec![0, 1, 1]), 0),
        QAdic::zero(2),
        QAdic::zero(2),
        QAdic::one(2),
    ];
    assert_eq!(Polynomial::new(pmonic_expected_coeffs), poly.monic());

    // 5-adic `7130 -> 1`
    assert_eq!(Polynomial::new_with_prime(5, vec![1]), Polynomial::<EAdic>::new_with_prime(5, vec![7130]).monic());

    // 5-adic `...12130 -> 1`
    assert_eq!(
        Polynomial::new_with_prime(5, vec![1]),
        Polynomial::new(vec![ZAdic::new_approx(5, 5, vec![0, 3, 1, 2, 1])]).monic()
    );

}

#[test]
fn variety() {
    let expected = Variety::new(vec![
        QAdic::from_integer(zadic_approx!(7, 3, [3, 1, 2])),
        QAdic::from_integer(zadic_approx!(7, 3, [4, 5, 4])),
    ]);
    let actual = EAdicPolynomial::new(vec![
        -eadic!(7, [2]),
        EAdic::zero(7),
        EAdic::one(7),
    ]).variety(3).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn display() {
    let expected = "1._5x^2 + 0._5x^1 + (4)._5x^0".to_string();
    let actual = EAdicPolynomial::new(vec![eadic_rep!(5, [], [4]), eadic_rep!(5, [], []), eadic_rep!(5, [1], [])]).to_string();

    assert_eq!(expected, actual);
}


// OPS

#[test]
fn add() {

    let lhs = EAdicPolynomial::new_with_prime(5, vec![-1, 1]);
    let rhs = EAdicPolynomial::new_with_prime(5, vec![-2, 1]);
    let result = lhs + rhs;
    let expected = EAdicPolynomial::new_with_prime(5, vec![-3, 2]);

    assert_eq!(result, expected);

    let lhs = EAdicPolynomial::new_with_prime(5, vec![-1, 0, 1]);
    let rhs = EAdicPolynomial::new_with_prime(5, vec![-2, 1]);
    let expected = EAdicPolynomial::new_with_prime(5, vec![-3, 1, 1]);

    assert_eq!(lhs.clone() + rhs.clone(), expected);
    assert_eq!(rhs.clone() + lhs.clone(), expected);

    let zero_poly = EAdicPolynomial::new(vec![]);
    assert_eq!(zero_poly.clone() + zero_poly.clone(), zero_poly.clone());
    assert_eq!(lhs.clone() + zero_poly.clone(), lhs.clone());
    assert_eq!(zero_poly.clone() + rhs.clone(), rhs.clone());
}

#[test]
#[should_panic]
fn add_mixed_characteristic() {
    let lhs = EAdicPolynomial::new_with_prime(5, vec![-1, 1]);
    let rhs = EAdicPolynomial::new_with_prime(7, vec![-2, 1]);
    let _ = lhs + rhs;
}

#[test]
fn mul() {
    // (x - 1) * (x - 2)
    // (x^2 - 3x + 2)
    let lhs = EAdicPolynomial::new_with_prime(5, vec![-1, 1]);
    let rhs = EAdicPolynomial::new_with_prime(5, vec![-2, 1]);
    let result = lhs * rhs;
    let expected = EAdicPolynomial::new_with_prime(5, vec![2, -3, 1]);

    assert_eq!(result, expected);

    // (x^2 - 1) * (x - 2)
    // (x^3 - 2x^2 - x + 2)
    let lhs = EAdicPolynomial::new_with_prime(5, vec![-1, 0, 1]);
    let rhs = EAdicPolynomial::new_with_prime(5, vec![-2, 1]);
    let expected = EAdicPolynomial::new_with_prime(5, vec![2, -1, -2, 1]);

    assert_eq!(lhs.clone() * rhs.clone(), expected);
    assert_eq!(rhs.clone() * lhs.clone(), expected);

    let zero = EAdicPolynomial::new(vec![]);
    assert_eq!(zero.clone() * zero.clone(), zero.clone());
    assert_eq!(lhs.clone() * zero.clone(), zero.clone());
    assert_eq!(zero.clone() * rhs.clone(), zero.clone());

    let one = EAdicPolynomial::new(vec![EAdic::one(5)]);
    assert_eq!(one.clone() * one.clone(), one.clone());
    assert_eq!(lhs.clone() * one.clone(), lhs.clone());
    assert_eq!(one.clone() * rhs.clone(), rhs.clone());
}

#[test]
#[should_panic]
fn mul_mixed_characteristic() {
    let lhs = EAdicPolynomial::new_with_prime(5, vec![-1, 1]);
    let rhs = EAdicPolynomial::new_with_prime(7, vec![-2, 1]);
    let _ = lhs * rhs;
}

#[test]
fn neg() {
    // Base case
    let poly = EAdicPolynomial::new_with_prime(5, vec![-1, 1]);
    let expected = EAdicPolynomial::new_with_prime(5, vec![1, -1]);
    assert_eq!(-poly.clone(), expected);

    // Double negative
    assert_eq!(poly.clone(), -(-poly.clone()));

    // Negative zero equals zero
    let zero = EAdicPolynomial::new(vec![]);
    assert_eq!(zero.clone(), -zero.clone());
}

#[test]
fn sub() {
    // Base case
    let lhs = EAdicPolynomial::new_with_prime(5, vec![1, 1]);
    let rhs = EAdicPolynomial::new_with_prime(5, vec![2]);
    let result = lhs - rhs;
    let expected = EAdicPolynomial::new_with_prime(5, vec![-1, 1]);
    assert_eq!(result, expected);

    // Subtract zero equals self
    let lhs = EAdicPolynomial::new_with_prime(5, vec![-1, 1]);
    let zero = EAdicPolynomial::new(vec![]);
    let result = lhs.clone() - zero.clone();
    assert_eq!(result, lhs.clone());

    // Zero minus zero
    assert_eq!(zero.clone() - zero.clone(), zero.clone());

    // Subtract leading degree
    let lhs = EAdicPolynomial::new_with_prime(5, vec![-1, 2, 1]);
    let rhs = EAdicPolynomial::new_with_prime(5, vec![0, 0, 1]);
    let result = lhs - rhs;
    let expected = EAdicPolynomial::new_with_prime(5, vec![-1, 2]);
    assert_eq!(result, expected);
}

#[test]
#[should_panic]
fn sub_mixed_characteristic() {
    let lhs = EAdicPolynomial::new_with_prime(5, vec![-1, 1]);
    let rhs = EAdicPolynomial::new_with_prime(7, vec![-2, 1]);
    let _ = lhs - rhs;
}

#[test]
fn derivative() {
    let actual = EAdicPolynomial::new(vec![
        eadic!(5, [1]),
        eadic_neg!(5, [2]),
        eadic!(5, [1]),
    ]).derivative();
    let expected = EAdicPolynomial::new(vec![
        eadic_neg!(5, [2]),
        eadic!(5, [2]),
    ]);
    assert_eq!(actual, expected);

    let actual = EAdicPolynomial::new(vec![
        eadic_rep!(5, [1], []),
        eadic_rep!(5, [], []),
        eadic_rep!(5, [4], [3]),
        eadic_rep!(5, [1], []),
    ]).derivative();
    let expected = EAdicPolynomial::new(vec![
        eadic_rep!(5, [], []),
        eadic_rep!(5, [3], [2]),
        eadic_rep!(5, [3], []),
    ]);
    assert_eq!(actual, expected);

    let actual = EAdicPolynomial::new(vec![
        eadic!(5, [1]),
        eadic!(5, []),
        eadic!(5, [1]),
    ]).derivative();
    let expected = EAdicPolynomial::new(vec![
        eadic!(5, []),
        eadic!(5, [2]),
    ]);
    assert_eq!(actual, expected);
}


// SQUARE FREE FACTORIZATION

#[test]
fn square_free() {

    // (x + 1) (x + 2)^2 (x + 3)^3
    let f = Polynomial::<EAdic>::new_with_prime(5, vec![108, 324, 387, 238, 80, 14, 1]);
    let sff = f.primitive_square_free_decomposition();
    assert_eq!(sff.len(), 3);
    assert_eq!(sff[0], Polynomial::new_with_prime(5, vec![1, 1]));
    assert_eq!(sff[1], Polynomial::new_with_prime(5, vec![2, 1]));
    assert_eq!(sff[2], Polynomial::new_with_prime(5, vec![3, 1]));

    // (x + 1) (x - 1) (x + 2)
    let f = Polynomial::<EAdic>::new_with_prime(3, vec![-2, -1, 1, 1]);
    let sff = f.primitive_square_free_decomposition();
    assert_eq!(sff.len(), 1);
    assert_eq!(sff[0], f);

    // (x + 1) (x + 2)^3
    let f = Polynomial::<EAdic>::new_with_prime(5, vec![8, 20, 18, 7, 1]);
    let sff = f.primitive_square_free_decomposition();
    assert_eq!(sff.len(), 3);
    assert_eq!(sff[0], Polynomial::new_with_prime(5, vec![1, 1]));
    assert!(sff[1].is_constant());
    assert_eq!(sff[1], Polynomial::new_with_prime(5, vec![1]));
    assert_eq!(sff[2], Polynomial::new_with_prime(5, vec![2, 1]));

    // (x + 1) (x^2 + 1)^2
    let f = Polynomial::<EAdic>::new_with_prime(7, vec![1, 1, 2, 2, 1, 1]);
    let sff = f.primitive_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert_eq!(sff[0], Polynomial::new_with_prime(7, vec![1, 1]));
    assert_eq!(sff[1], Polynomial::new_with_prime(7, vec![1, 0, 1]));

    // (x - 1)^2 (x - 26)^2 = (x^2 - 27 x + 26)^2 = x^4 - 54 x^3 + 781 x^2 - 1404 x + 676
    let f = Polynomial::<EAdic>::new_with_prime(5, vec![676, -1404, 781, -54, 1]);
    let sff = f.primitive_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert!(sff[0].is_constant());
    assert_eq!(sff[1], Polynomial::new_with_prime(5, vec![26, -27, 1]));

}

#[test]
fn approx_square_free() {

    let p = 5;

    // f(x) = (x - 1) (x - ...1111) = x^2 - ...1112 x + ...1111 = x^2 + ...3333 x + ...1111
    let f = Polynomial::<ZAdic>::new(vec![
        ZAdic::new_approx(p, 4, vec![1, 1, 1, 1]),
        ZAdic::new_approx(p, 4, vec![3, 3, 3, 3]),
        ZAdic::one(p),
    ]);
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(1, sff.len());
    assert_eq!(f, sff[0]);

    // x^3 + 4x^2 - 4x - 16
    // (x - 2)(x + 2)(x + 4)
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![-16, -4, 4, 1]);
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(1, sff.len());

    // (x + 1)^2 = x^2 + 2x + 1
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![1, 2, 1]);
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(2, sff.len());

    // (x + 1)^2 (x + 2) = x^3 + 4x^2 + 5x + 2
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![2, 5, 4, 1]);
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(2, sff.len());

    // (5x - 1)^2 = 25x^2 - 10x + 1
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![1, -10, 25]);
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(2, sff.len());

    // (x - 1)^2 (x - 26)^2 = x^4 - 54 x^3 + 781 x^2 - 1404 x + 676
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![676, -1404, 781, -54, 1]);
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(2, sff.len());

    // (5x - 1)^2 (5x - 26)^2 = 625x^4 - 6750 x^3 + 19525 x^2 - 7020 x + 676
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![676, -7020, 19525, -6750, 625]);
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(2, sff.len());

    // ZAdic (x + 1) (x + 2)^2 (x + 3)^3
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![108, 324, 387, 238, 80, 14, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(10));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 3);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![zadic_approx!(p, 9, [1]), ZAdic::one(p)])
    );
    assert_eq!(
        sff[1],
        Polynomial::new(vec![zadic_approx!(p, 9, [2]), ZAdic::one(p)])
    );
    assert_eq!(
        sff[2],
        Polynomial::new(vec![zadic_approx!(p, 9, [3]), ZAdic::one(p)])
    );

    // (5x - 1)^2 (5x - 26)^2 = 625x^4 - 6750 x^3 + 19525 x^2 - 7020 x + 676
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![676, -7020, 19525, -6750, 625]);
    let f = f.map_coefficients(|c| c.into_approximation(16));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert!(sff[0].is_constant());
    assert_eq!(
        sff[1],
        Polynomial::new(vec![
            ZAdic::primed_from(p, 26).into_approximation(8),
            ZAdic::primed_from(p, -135).into_approximation(9),
            ZAdic::primed_from(p, 25)
        ])
    );

    // WRONG for precision 8, right for precision 9
    // (x - 5)^2 (x - 30)^2 = x^4 - 70 x^3 + 1525 x^2 - 10500 x + 22500
    let precision = 8;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![22500, -10500, 1525, -70, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    // PRECISION 9
    // assert_eq!(sff.len(), 2);
    // assert!(sff[0].is_constant());
    // assert_eq!(
    //     sff[1],
    //     Polynomial::new(vec![
    //         ZAdic::primed_from(p, 150).into_approximation(precision-4),
    //         ZAdic::primed_from(p, -35).into_approximation(precision-4),
    //         ZAdic::primed_from(p, 1)
    //     ])
    // );
    // PRECISION 8
    assert_eq!(sff.len(), 3);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::primed_from(p, 295).into_approximation(precision-4),
            ZAdic::primed_from(p, 1)
        ])
    );
    assert!(sff[1].is_constant());
    assert_eq!(
        sff[2],
        Polynomial::new(vec![
            ZAdic::primed_from(p, 295).into_approximation(precision-4),
            ZAdic::primed_from(p, 1)
        ])
    );

    // WRONG for precision 12, right for precision 13
    // (x - 5)^2 (x - 130)^2 = x^4 - 270 x^3 + 19525 x^2 - 175500 x + 422500
    let precision = 12;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![422500, -175500, 19525, -270, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    // PRECISION 13
    // assert_eq!(sff.len(), 2);
    // assert!(sff[0].is_constant());
    // assert_eq!(
    //     sff[1],
    //     Polynomial::new(vec![
    //         ZAdic::primed_from(p, 650).into_approximation(precision-6),
    //         ZAdic::primed_from(p, -135).into_approximation(precision-6),
    //         ZAdic::primed_from(p, 1)
    //     ])
    // );
    // PRECISION 12
    assert_eq!(sff.len(), 3);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::new_approx(p, 6, vec![0, 4, 4, 1, 2, 2]),
            ZAdic::primed_from(p, 1)
        ])
    );
    assert!(sff[1].is_constant());
    assert_eq!(
        sff[2],
        Polynomial::new(vec![
            ZAdic::new_approx(p, 6, vec![0, 4, 4, 1, 2, 2]),
            ZAdic::primed_from(p, 1)
        ])
    );

    // WRONG for precision 20, right for precision 21
    // (x - 125)^2 (x - 3250)^2 = x^4 - 6750 x^3 + 12203125 x^2 - 2742187500 x + 165039062500
    let precision = 20;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![
        BigInt::from_str_radix("165039062500", 10).unwrap(),
        BigInt::from_str_radix("-2742187500", 10).unwrap(),
        BigInt::from(12203125),
        BigInt::from(-6750),
        BigInt::from(1)
    ]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    // PRECISION 21
    // assert_eq!(sff.len(), 2);
    // assert!(sff[0].is_constant());
    // assert_eq!(
    //     sff[1],
    //     Polynomial::new(vec![
    //         ZAdic::primed_from(p, 406250).into_approximation(precision-10),
    //         ZAdic::primed_from(p, -3375).into_approximation(precision-10),
    //         ZAdic::primed_from(p, 1)
    //     ])
    // );
    // PRECISION 20
    assert_eq!(sff.len(), 3);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::new_approx(p, 10, vec![0, 0, 0, 4, 4, 1, 2, 2, 2, 2, 2]),
            ZAdic::primed_from(p, 1)
        ])
    );
    assert!(sff[1].is_constant());
    assert_eq!(
        sff[2],
        Polynomial::new(vec![
            ZAdic::new_approx(p, 10, vec![0, 0, 0, 4, 4, 1, 2, 2, 2, 2, 2]),
            ZAdic::primed_from(p, 1)
        ])
    );

}

#[test]
fn square_free_precision_loss() {

    let p = 5;

    // NO PRECISION LOSS
    // ZAdic (x - 1)^2 (x - 2) = x^3 - 9 x^2 + 15 x - 7
    let precision = 8;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![-7, 15, -9, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::primed_from(p, -7).into_approximation(precision),
            ZAdic::one(p),
        ])
    );
    assert_eq!(
        sff[1],
        Polynomial::new(vec![
            ZAdic::primed_from(p, -1).into_approximation(precision),
            ZAdic::one(p),
        ])
    );

    // NO PRECISION LOSS
    // ZAdic (x - 1)^2 (x - 2)^2 = = (x^2 - 3x + 2)^2 = x^4 - 6 x^3 + 13 x^2 - 12 x + 4
    let precision = 8;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![4, -12, 13, -6, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert!(sff[0].is_constant());
    assert_eq!(
        sff[1],
        Polynomial::new(vec![
            ZAdic::primed_from(p, 2).into_approximation(precision),
            ZAdic::primed_from(p, -3).into_approximation(precision),
            ZAdic::one(p),
        ])
    );

    // NO PRECISION LOSS
    // ZAdic (x - 1) (x - 6) = x^2 - 7 x + 6
    let precision = 8;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![6, -7, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 1);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::primed_from(p, 6).into_approximation(precision),
            ZAdic::primed_from(p, -7).into_approximation(precision),
            ZAdic::one(p),
        ])
    );

    // PRECISION LOSS
    // ZAdic (x - 1)^2 (x - 6) = x^3 - 8 x^2 + 13 x - 6
    let precision = 8;
    let lower_precision = precision - 2;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![-6, 13, -8, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::primed_from(p, -6).into_approximation(lower_precision),
            ZAdic::one(p),
        ])
    );
    assert_eq!(
        sff[1],
        Polynomial::new(vec![
            ZAdic::primed_from(p, -1).into_approximation(lower_precision),
            ZAdic::one(p),
        ])
    );

    // PRECISION LOSS
    // ZAdic (x - 1)^2 (x - 6)^2 = (x^2 - 7 x + 6)^2 = x^4 - 14 x^3 + 61 x^2 - 84 x + 36
    let precision = 8;
    let lower_precision = precision - 2;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![36, -84, 61, -14, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert!(sff[0].is_constant());
    assert_eq!(
        sff[1],
        Polynomial::new(vec![
            ZAdic::primed_from(p, 6).into_approximation(lower_precision),
            ZAdic::primed_from(p, -7).into_approximation(lower_precision),
            ZAdic::one(p),
        ])
    );

    // PRECISION LOSS
    // ZAdic (x - 1)^2 (x - 26) = x^3 - 28 x^2 + 53 x - 26
    let precision = 8;
    let lower_precision = precision - 4;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![-26, 53, -28, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::primed_from(p, -26).into_approximation(lower_precision),
            ZAdic::one(p),
        ])
    );
    assert_eq!(
        sff[1],
        Polynomial::new(vec![
            ZAdic::primed_from(p, -1).into_approximation(lower_precision),
            ZAdic::one(p),
        ])
    );

    // PRECISION LOSS AND OVERLAP
    // ZAdic (x - 1)^2 (x - 6)^2 = (x^2 - 7 x + 6)^2 = x^4 - 14 x^3 + 61 x^2 - 84 x + 36

    // PRECISION 4 => (x + ...14) * (x + ...14)
    let precision = 4;
    let lower_precision = precision - 2;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![36, -84, 61, -14, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 3);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::new_approx(p, lower_precision, vec![4, 1]),
            ZAdic::one(p),
        ])
    );
    assert!(sff[1].is_constant());
    assert_eq!(
        sff[2],
        Polynomial::new(vec![
            ZAdic::new_approx(p, lower_precision, vec![4, 1]),
            ZAdic::one(p),
        ])
    );
    // PRECISION 5 => (x^2 - 7 x + 6)^2
    let precision = 5;
    let lower_precision = precision - 2;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![36, -84, 61, -14, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert!(sff[0].is_constant());
    assert_eq!(
        sff[1],
        Polynomial::new(vec![
            ZAdic::primed_from(p, 6).into_approximation(lower_precision),
            ZAdic::primed_from(p, -7).into_approximation(lower_precision),
            ZAdic::one(p),
        ])
    );

    // PRECISION LOSS AND OVERLAP
    // ZAdic (x - 1)^2 (x - 26) = x^3 - 28 x^2 + 53 x - 26

    // PRECISION 4 => (x + ...1244)^3
    let precision = 4;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![-26, 53, -28, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 3);
    assert!(sff[0].is_constant());
    assert!(sff[1].is_constant());
    assert_eq!(
        sff[2],
        Polynomial::new(vec![
            ZAdic::new_approx(p, precision, vec![4, 4, 2, 1]),
            ZAdic::one(p),
        ])
    );
    // PRECISION 5 => (x - 26) * (x - 1)^2
    let precision = 5;
    let lower_precision = precision - 4;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![-26, 53, -28, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::new_approx(p, lower_precision, vec![4]),
            ZAdic::one(p),
        ])
    );
    assert_eq!(
        sff[1],
        Polynomial::new(vec![
            ZAdic::new_approx(p, lower_precision, vec![4]),
            ZAdic::one(p),
        ])
    );

    // PRECISION LOSS AND OVERLAP
    // ZAdic (x - 1)^2 (x - 26)^2 = (x^2 - 27 x + 26)^2 = x^4 - 54 x^3 + 781 x^2 - 1404 x + 676

    // PRECISION 8 => (x + ...2144) * (x + ...2144)^3
    let precision = 8;
    let lower_precision = precision - 4;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![676, -1404, 781, -54, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 3);
    assert_eq!(
        sff[0],
        Polynomial::new(vec![
            ZAdic::new_approx(p, lower_precision, vec![4, 4, 1, 2]),
            ZAdic::one(p),
        ])
    );
    assert!(sff[1].is_constant());
    assert_eq!(
        sff[2],
        Polynomial::new(vec![
            ZAdic::new_approx(p, lower_precision, vec![4, 4, 1, 2]),
            ZAdic::one(p),
        ])
    );
    // PRECISION 9 => (x^2 - 27 x + 26)^2
    let precision = 9;
    let lower_precision = precision - 4;
    let f = Polynomial::<EAdic>::new_with_prime(p, vec![676, -1404, 781, -54, 1]);
    let f = f.map_coefficients(|c| c.into_approximation(precision));
    let sff = f.zadic_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert!(sff[0].is_constant());
    assert_eq!(
        sff[1],
        Polynomial::new(vec![
            ZAdic::primed_from(p, 26).into_approximation(lower_precision),
            ZAdic::primed_from(p, -27).into_approximation(lower_precision),
            ZAdic::one(p),
        ])
    );

}


// SUBRESULTANT (i.e. GCD internals)

use euclid::PolynomialSubresultant;

#[test]
fn eadic_subresultant() {

    // x^2 + 2x + 1
    let a = Polynomial::<EAdic>::new_with_prime(5, vec![1, 2, 1]);
    // 2x + 2
    let b = a.derivative();

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert!(subresultant.is_trivial());

    // Primitive gcd
    let subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(Polynomial::<EAdic>::new_with_prime(5, vec![2, 2]), subresultant.into_factored_gcd());

    // x^2 + 3x + 1
    let a = Polynomial::<EAdic>::new_with_prime(5, vec![1, 3, 1]);
    // 2x + 3
    let b = a.derivative();

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert_eq!(Polynomial::<EAdic>::new_with_prime(5, vec![-1]), subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert!(subresultant.is_trivial());

    // Primitive gcd
    let subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(Polynomial::<EAdic>::new_with_prime(5, vec![-1]), subresultant.into_factored_gcd());

    // (x+1)^2(x^2-7) = x^4 + 2x^3 - 6x^2 - 14x - 7
    let a = Polynomial::<EAdic>::new_with_prime(5, vec![-7, -14, -6, 2, 1]);
    // (x+1)(x+2)^2(x+3) = x^4 + 8x^3 + 23x^2 + 28x + 12
    let b = Polynomial::<EAdic>::new_with_prime(5, vec![12, 28, 23, 8, 1]);

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());

    // Primitive gcd
    assert_eq!(
        Polynomial::<EAdic>::new_with_prime(5, vec![-1296, -1296]),
        subresultant.clone().into_factored_gcd()
    );
    assert_eq!(
        Polynomial::<EAdic>::new_with_prime(5, vec![1, 1]),
        subresultant.clone().into_factored_gcd().primitive_monic()
    );
    assert_eq!(
        Polynomial::<EAdic>::new_with_prime(5, vec![1, 1]),
        subresultant.clone().into_primitive_gcd()
    );
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert_eq!(Polynomial::<EAdic>::new_with_prime(5, vec![-19, -42, -29, -6]), subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert_eq!(Polynomial::<EAdic>::new_with_prime(5, vec![71, 96, 25]), subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert_eq!(Polynomial::<EAdic>::new_with_prime(5, vec![-1296, -1296]), subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert!(subresultant.is_trivial());

}

#[test]
fn zadic_subresultant() {

    // GCD certainty can be lower than polynomial certainty
    let p = 5;
    let precision = 8;

    // Sometimes it is NOT: gcd((x-1)^2, (x-1)(x-2)) = x-1 to SAME precision
    let a = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 1).into_approximation(precision),
        ZAdic::primed_from(p, -2).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let b = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 2).into_approximation(precision),
        ZAdic::primed_from(p, -3).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let expected = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, -1).into_approximation(precision),
        ZAdic::one(p),
    ]));

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert_eq!(
        zadic_wrapper::wrap_poly(Polynomial::new(vec![
            ZAdic::primed_from(p, -1).into_approximation(precision),
            ZAdic::primed_from(p, 1).into_approximation(precision),
        ])),
        subresultant.poly1
    );
    subresultant = subresultant.next_factored();
    assert!(subresultant.is_trivial());

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert_eq!(
        zadic_wrapper::wrap_poly(Polynomial::new(vec![
            ZAdic::primed_from(p, -1).into_approximation(precision),
            ZAdic::primed_from(p, 1).into_approximation(precision),
        ])),
        subresultant.poly1
    );
    subresultant = subresultant.next_factored();
    assert!(subresultant.is_trivial());

    // Sometimes it is NOT: gcd((x-1)^2, (x-1)(x-5)) = x-1 to SAME precision
    let a = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 1).into_approximation(precision),
        ZAdic::primed_from(p, -2).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let b = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 5).into_approximation(precision),
        ZAdic::primed_from(p, -6).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let expected = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, -1).into_approximation(precision),
        ZAdic::one(p),
    ]));

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert_eq!(
        zadic_wrapper::wrap_poly(Polynomial::new(vec![
            ZAdic::primed_from(p, -4).into_approximation(precision),
            ZAdic::primed_from(p, 4).into_approximation(precision),
        ])),
        subresultant.poly1
    );
    subresultant = subresultant.next_factored();
    assert!(subresultant.is_trivial());

    // Sometimes it IS: gcd((x-1)^2, (x-1)(x-6)) = x-1 to LOWER precision
    let lower_precision = precision - 1;
    let a = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 1).into_approximation(precision),
        ZAdic::primed_from(p, -2).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let b = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 6).into_approximation(precision),
        ZAdic::primed_from(p, -7).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let expected = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, -1).into_approximation(lower_precision),
        ZAdic::one(p),
    ]));

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert_eq!(
        zadic_wrapper::wrap_poly(Polynomial::new(vec![
            ZAdic::primed_from(p, -1).into_approximation(lower_precision),
            ZAdic::primed_from(p, 1).into_approximation(lower_precision),
        ])),
        subresultant.poly1
    );
    subresultant = subresultant.next_factored();
    assert!(subresultant.is_trivial());

    // Sometimes it IS: gcd((x-1)(x-2), (x-1)(x-7)) = x-1 to LOWER precision
    let lower_precision = precision - 1;
    let a = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 2).into_approximation(precision),
        ZAdic::primed_from(p, -3).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let b = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 7).into_approximation(precision),
        ZAdic::primed_from(p, -8).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let expected = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, -1).into_approximation(lower_precision),
        ZAdic::one(p),
    ]));

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next_factored();
    assert_eq!(
        zadic_wrapper::wrap_poly(Polynomial::new(vec![
            ZAdic::primed_from(p, -1).into_approximation(lower_precision),
            ZAdic::primed_from(p, 1).into_approximation(lower_precision),
        ])),
        subresultant.poly1
    );
    subresultant = subresultant.next_factored();
    assert!(subresultant.is_trivial());

}

#[test]
fn zadic_precision_overlap() {

    // gcd(f, f'), f = (x - 1)^2 (x - 26) = x^3 - 28 x^2 + 53 x - 26

    // f = x^3 - 28 x^2 + 53 x - 26
    // f' = 3 x^2 - 56 x + 53

    // "WRONG": PRECISION 4 => (x + ...1244)^2
    // "Because" (x + ...1244)^3 = x^3 + ...4342 x^2 + ...0203 x + ...4344 = f.into_approximation(4).primitive_monic()
    let p = 5;
    let precision = 4;
    let a = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, -26).into_approximation(precision),
        ZAdic::primed_from(p, 53).into_approximation(precision),
        ZAdic::primed_from(p, -28).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let b = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 53).into_approximation(precision),
        ZAdic::primed_from(p, -56).into_approximation(precision),
        ZAdic::primed_from(p, 3)
    ]));
    // let expected = zadic_wrapper::wrap_poly(Polynomial::new(vec![
    //     ZAdic::primed_from(p, -1).into_approximation(precision),
    //     ZAdic::one(p),
    // ]));
    let expected = b.clone().primitive_monic();

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next();
    assert!(subresultant.is_trivial());

    // RIGHT BUT WITH LOW PRECISION: PRECISION 5 => (x - ...1)
    let p = 5;
    let precision = 5;
    let lower_precision = precision - 4;
    let a = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, -26).into_approximation(precision),
        ZAdic::primed_from(p, 53).into_approximation(precision),
        ZAdic::primed_from(p, -28).into_approximation(precision),
        ZAdic::one(p),
    ]));
    let b = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 53).into_approximation(precision),
        ZAdic::primed_from(p, -56).into_approximation(precision),
        ZAdic::primed_from(p, 3)
    ]));
    let expected = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, -1).into_approximation(lower_precision),
        ZAdic::one(p),
    ]));

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next();
    assert_eq!(expected, subresultant.poly1.clone().primitive_monic());
    subresultant = subresultant.next();
    assert!(subresultant.is_trivial());

    // (5x - 1)^2 (5x - 26)^2 = 625x^4 - 6750 x^3 + 19525 x^2 - 7020 x + 676
    // (constant coefficient).into_approximation(15)
    // 10000._5x^4 + (4)241000._5x^3 + 1111100._5x^2 + (4)233410._5x^1 + ...10201._5x^0
    let p = 5;
    let precision = 5;
    let f = Polynomial::new(vec![
        ZAdic::primed_from(p, 676).into_approximation(precision),
        ZAdic::primed_from(p, -7020),
        ZAdic::primed_from(p, 19525),
        ZAdic::primed_from(p, -6750),
        ZAdic::primed_from(p, 625),
    ]);
    let a = zadic_wrapper::wrap_poly(f.clone());
    let b = zadic_wrapper::wrap_poly(f.derivative());
    let expected = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 26).into_approximation(precision-4),
        ZAdic::primed_from(p, -135),
        ZAdic::primed_from(p, 25),
    ]));

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next();
    assert_eq!(expected, subresultant.poly1.clone().primitive_monic());
    subresultant = subresultant.next();
    assert!(subresultant.is_trivial());

    // (x - 125)^2 (x - 3250)^2 = x^4 - 6750 x^3 + 12203125 x^2 - 2742187500 x + 165039062500
    // (constant coefficient).into_approximation(15)
    // 1._5x^4 + (4)241000._5x^3 + 11111000000._5x^2 + (4)23341000000000._5x^1 + ...201000000000000._5x^0
    let p = 5;
    let precision = 15;
    let f = Polynomial::new(vec![
        ZAdic::primed_from(p, BigInt::from_str_radix("165039062500", 10).unwrap()).into_approximation(precision),
        ZAdic::primed_from(p, BigInt::from_str_radix("-2742187500", 10).unwrap()),
        ZAdic::primed_from(p, 12203125),
        ZAdic::primed_from(p, -6750),
        ZAdic::primed_from(p, 1),
    ]);
    let a = zadic_wrapper::wrap_poly(f.clone());
    let b = zadic_wrapper::wrap_poly(f.derivative());
    let expected = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 406250).into_approximation(precision-10),
        ZAdic::primed_from(p, -3375),
        ZAdic::primed_from(p, 1),
    ]));

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next();
    assert_eq!(expected, subresultant.poly1.clone().primitive_monic());
    subresultant = subresultant.next();
    assert!(subresultant.is_trivial());

    // (x - 125)^2 (x - 3250)^2 = x^4 - 6750 x^3 + 12203125 x^2 - 2742187500 x + 165039062500
    // (leading coefficient).into_approximation(15)
    // ...000000000000001._5x^4 + (4)241000._5x^3 + 11111000000._5x^2 + (4)23341000000000._5x^1 + 10201000000000000._5x^0
    let p = 5;
    let precision = 15;
    let f = Polynomial::new(vec![
        ZAdic::primed_from(p, BigInt::from_str_radix("165039062500", 10).unwrap()),
        ZAdic::primed_from(p, BigInt::from_str_radix("-2742187500", 10).unwrap()),
        ZAdic::primed_from(p, 12203125),
        ZAdic::primed_from(p, -6750),
        ZAdic::primed_from(p, 1).into_approximation(precision),
    ]);
    let a = zadic_wrapper::wrap_poly(f.clone());
    let b = zadic_wrapper::wrap_poly(f.derivative());
    let expected = zadic_wrapper::wrap_poly(Polynomial::new(vec![
        ZAdic::primed_from(p, 406250).into_approximation(precision+2),
        ZAdic::primed_from(p, -3375).into_approximation(precision-1),
        ZAdic::primed_from(p, 1),
    ]));

    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(subresultant.clone().into_primitive_gcd(), expected);
    assert_eq!(b, subresultant.poly1);
    subresultant = subresultant.next();
    assert_eq!(expected, subresultant.poly1.clone().primitive_monic());
    subresultant = subresultant.next();
    assert!(subresultant.is_trivial());

}


// NEWTON POLYGON

#[test]
fn newton_polygon() {

    use Valuation::{Finite, PosInf};

    let coeffs = vec![Rational32::one(), Rational32::zero(), Rational32::one(), Rational32::new(1, 3), Rational32::new(3, 1)];
    let poly = Polynomial::<QAdic<EAdic>>::new_with_prime(3, coeffs.clone());
    let newton_polygon = NewtonPolygon::from_polynomial(&poly);
    assert_eq!(
        vec![(0, Finite(0isize)), (3, Finite(-1)), (4, Finite(1)), (5, PosInf)],
        newton_polygon.vertices().collect::<Vec<_>>()
    );

    let front_zero_coeffs = [vec![Rational32::zero()], coeffs.clone()].concat();
    let poly = Polynomial::<QAdic<EAdic>>::new_with_prime(3, front_zero_coeffs);
    let newton_polygon = NewtonPolygon::from_polynomial(&poly);
    assert_eq!(
        vec![(0, PosInf), (1, Finite(0isize)), (4, Finite(-1)), (5, Finite(1)), (6, PosInf)],
        newton_polygon.vertices().collect::<Vec<_>>()
    );

    let back_zero_coeffs = [coeffs.clone(), vec![Rational32::zero()]].concat();
    let poly = Polynomial::<QAdic<EAdic>>::new_with_prime(3, back_zero_coeffs);
    let newton_polygon = NewtonPolygon::from_polynomial(&poly);
    assert_eq!(
        vec![(0, Finite(0isize)), (3, Finite(-1)), (4, Finite(1)), (5, PosInf)],
        newton_polygon.vertices().collect::<Vec<_>>()
    );

}
