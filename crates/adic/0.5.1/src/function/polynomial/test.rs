use num::Zero;

use crate::mapping::{Differentiable, IndexedMapping, Mapping};
use super::{euclid, Polynomial};


#[test]
fn new() {
    Polynomial::new(vec![1]);
    Polynomial::new(vec![1.0, 2.0]);
    assert_eq!(Polynomial::new(vec![0, 0, 4]), Polynomial::monomial(2, 4));
}

#[test]
fn degree() {
    assert_eq!(Polynomial::new(vec![-2, 0, 1]).degree(), Some(2));
    assert_eq!(Polynomial::new(vec![-1, 1]).degree(), Some(1));
    assert_eq!(Polynomial::new(vec![1]).degree(), Some(0));
    assert_eq!(Polynomial::<u32>::new(vec![]).degree(), None);
}

#[test]
fn display() {
    let expected = "1x^2 + 0x^1 + -1x^0".to_string();
    let actual = Polynomial::new(vec![-1, 0, 1]).to_string();
    assert_eq!(expected, actual);
}

#[test]
fn compose() {
    let f = Polynomial::new(vec![1, 1]);
    let g = Polynomial::new(vec![0, 0, 1]);
    assert_eq!(f.clone().compose(g.clone()).eval(2), Polynomial::new(vec![1, 0, 1]).eval(2));
    assert_eq!(g.compose(f.clone()).eval(2), Polynomial::new(vec![1, 2, 1]).eval(2));
    let f =|x| 1.0 / x;
    let g = Polynomial::new(vec![0.0, 0.0, 1.0]);
    assert_eq!(f.clone().compose(g.clone()).eval(2.0), Ok(0.25));
}

#[test]
fn eval_finite() {
    let f = Polynomial::new(vec![1, 1, 1, 1]);
    assert_eq!(f.eval_finite(3, 1), Ok(1));
    assert_eq!(f.eval_finite(3, 2), Ok(4));
    assert_eq!(f.eval_finite(3, 3), Ok(13));
    assert_eq!(f.eval_finite(3, 4), Ok(40));
    assert_eq!(f.eval_finite(3, 4), f.eval(3));
}

#[test]
fn ops() {

    let f0 = Polynomial::new(vec![6, 7, 1]);
    let f1 = Polynomial::new(vec![-6, -7, 1]);
    assert_eq!(f0.clone()+f1.clone(), Polynomial::new(vec![0, 0, 2]));
    assert_eq!(f0.clone()-f1.clone(), Polynomial::new(vec![12, 14]));
    assert_eq!(-f0.clone(), Polynomial::new(vec![-6, -7, -1]));
    assert_eq!(-f1.clone(), Polynomial::new(vec![6, 7, -1]));
    assert_eq!(f0.clone()+f1.clone(), -((-f0.clone()) + (-f1.clone())));
    assert_eq!(f0.clone()*f1.clone(), Polynomial::new(vec![-36, -84, -49, 0, 1]));

    assert_eq!(f0.derivative(), Polynomial::new(vec![7, 2]));
    assert_eq!(f1.derivative(), Polynomial::new(vec![-7, 2]));
    assert_eq!((f0.clone()+f1.clone()).derivative(), f0.derivative() + f1.derivative());
    assert_eq!((f0.clone()*f1.clone()).derivative(), f0.clone()*f1.derivative() + f0.derivative()*f1.clone());

}

#[test]
fn gcd() {

    // gcd(x^2 + 7 x + 6, x^2 - 5 x - 6) = 12 x + 12
    let f0 = Polynomial::new(vec![6, 7, 1]);
    let f1 = Polynomial::new(vec![-6, -5, 1]);
    let g = Polynomial::new(vec![1, 1]);
    assert_eq!(g, f0.gcd(&f1));
    assert_eq!(g, -f1.gcd(&f0));

    // gcd(a, 0) = a, gcd(0, b) = b
    let zero = Polynomial::zero();
    assert_eq!(f1, zero.gcd(&f1));
    assert_eq!(f0, f0.gcd(&zero));
    assert_eq!(zero, zero.gcd(&zero));

    // gcd(x^4 - 1, x - 1) = x - 1
    let f0 = Polynomial::new(vec![-1, 0, 0, 0, 1]);
    let f1 = Polynomial::new(vec![-1, 1]);
    assert_eq!(f1, f0.gcd(&f1));
    assert_eq!(f1, f1.gcd(&f0));

    // gcd(x^8 + x^6 - 3 x^4 - 3 x^3 + 8 x^2 + 2 x - 5, 3 x^6 + 5 x^4 - 4 x^2 - 9 x + 21) = const
    let f0 = Polynomial::new(vec![-5, 2, 8, -3, -3, 0, 1, 0, 1]);
    let f1 = Polynomial::new(vec![21, -9, -4, 0, 5, 0, 3]);
    let g = Polynomial::new(vec![1]);
    assert_eq!(g, -f0.gcd(&f1));
    assert_eq!(g, -f1.gcd(&f0));

    // gcd(81 x^4 - 16, 3 x^2 + 2 x) ~= 3 x + 2
    let f0 = Polynomial::new(vec![-16, 0, 0, 0, 81]);
    let f1 = Polynomial::new(vec![0, 2, 3]);
    let g = Polynomial::new(vec![2, 3]);
    assert_eq!(g, -f0.gcd(&f1));
    assert_eq!(g, -f1.gcd(&f0));
    let g = Polynomial::new(vec![2, 3]);
    assert_eq!(g, f0.gcd(&f1).primitive_monic());
    // gcd(81 x^4 - 16, 3 x + 2) ~= 3 x + 2
    let f0 = Polynomial::new(vec![-16, 0, 0, 0, 81]);
    let f1 = Polynomial::new(vec![2, 3]);
    let g = Polynomial::new(vec![2, 3]);
    assert_eq!(g, f0.gcd(&f1));
    assert_eq!(g, f1.gcd(&f0));
    // gcd(81 x^4 - 16, 3 x^2 - 2 x) ~= 3 x - 2
    let f1 = Polynomial::new(vec![0, -2, 3]);
    let g = Polynomial::new(vec![-2, 3]);
    assert_eq!(g, f0.gcd(&f1));
    assert_eq!(g, f1.gcd(&f0));
    let g = Polynomial::new(vec![-2, 3]);
    assert_eq!(g, f0.gcd(&f1).primitive_monic());
    // gcd(81 x^4 - 16, 3 x - 2) ~= 3 x - 2
    let f0 = Polynomial::new(vec![-16, 0, 0, 0, 81]);
    let f1 = Polynomial::new(vec![-2, 3]);
    let g = Polynomial::new(vec![-2, 3]);
    assert_eq!(g, f0.gcd(&f1));
    assert_eq!(g, f1.gcd(&f0));
    // gcd(81 x^4 - 16, 9 x^4 + 4 x^2) ~= 9 x^2 + 4
    let f1 = Polynomial::new(vec![0, 0, 4, 0, 9]);
    let g = Polynomial::new(vec![4, 0, 9]);
    assert_eq!(g, -f0.gcd(&f1));
    assert_eq!(g, f1.gcd(&f0));
    let g = Polynomial::new(vec![4, 0, 9]);
    assert_eq!(g, f0.gcd(&f1).primitive_monic());
    // gcd(81 x^4 - 16, 9 x^3 + 4 x) ~= 9 x^2 + 4
    let f1 = Polynomial::new(vec![0, 4, 0, 9]);
    let g = Polynomial::new(vec![4, 0, 9]);
    assert_eq!(g, -f0.gcd(&f1));
    assert_eq!(g, -f1.gcd(&f0));
    let g = Polynomial::new(vec![4, 0, 9]);
    assert_eq!(g, f0.gcd(&f1).primitive_monic());
    // gcd(81 x^4 - 16, 9 x^2 + 4) ~= 9 x^2 + 4
    let f1 = Polynomial::new(vec![4, 0, 9]);
    let g = Polynomial::new(vec![4, 0, 9]);
    assert_eq!(g, f0.gcd(&f1));
    assert_eq!(g, f1.gcd(&f0));

}

#[test]
fn square_free() {

    // (x + 1) (x + 2)^2 (x + 3)^3
    let f = Polynomial::new_big(vec![108, 324, 387, 238, 80, 14, 1]);
    let sff = f.raw_square_free_decomposition();
    assert_eq!(sff.len(), 3);
    assert_eq!(sff[0], Polynomial::new_big(vec![-1, -1]));
    assert_eq!(sff[1], Polynomial::new_big(vec![2, 1]));
    assert_eq!(sff[2], Polynomial::new_big(vec![3, 1]));

    // (x + 1) (x - 1) (x + 2)
    let f = Polynomial::new_big(vec![-2, -1, 1, 1]);
    let sff = f.raw_square_free_decomposition();
    assert_eq!(sff.len(), 1);
    assert_eq!(sff[0], f);

    // (x + 1) (x + 2)^3
    let f = Polynomial::new_big(vec![8, 20, 18, 7, 1]);
    let sff = f.raw_square_free_decomposition();
    assert_eq!(sff.len(), 3);
    assert_eq!(sff[0], Polynomial::new_big(vec![1, 1]));
    assert!(sff[1].is_constant());
    assert_eq!(sff[1], Polynomial::new_big(vec![1]));
    assert_eq!(sff[2], Polynomial::new_big(vec![2, 1]));

    // (x + 1) (x^2 + 1)^2
    let f = Polynomial::new_big(vec![1, 1, 2, 2, 1, 1]);
    let sff = f.raw_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert_eq!(sff[0], Polynomial::new_big(vec![1, 1]));
    assert_eq!(sff[1], Polynomial::new_big(vec![1, 0, 1]));

    // (x - 1)^2 (x - 26)^2 = (x^2 - 27 x + 26)^2 = x^4 - 54 x^3 + 781 x^2 - 1404 x + 676
    let f = Polynomial::new_big(vec![676, -1404, 781, -54, 1]);
    let sff = f.raw_square_free_decomposition();
    assert_eq!(sff.len(), 2);
    assert!(sff[0].is_constant());
    assert_eq!(sff[1], Polynomial::new_big(vec![26, -27, 1]));

}

#[test]
fn subresultant() {

    use euclid::PolynomialSubresultant;

    // x^8 + x^6 - 3x4 - 3x^3 + 8x^2 + 2x - 5
    let a = Polynomial::new(vec![-5, 2, 8, -3, -3, 0, 1, 0, 1]);
    // 3x^6 + 5x^4 - 4x^2 - 9x + 21
    let b = Polynomial::new(vec![21, -9, -4, 0, 5, 0, 3]);

    // Non-primitive subresultants
    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(Polynomial::new(vec![21, -9, -4, 0, 5, 0, 3]), subresultant.poly1);
    subresultant = subresultant.next();
    assert_eq!(Polynomial::new(vec![-3, 0, 1, 0, -5]), subresultant.poly1);
    subresultant = subresultant.next();
    assert_eq!(Polynomial::new(vec![441, -225, -117]), subresultant.poly1);
    subresultant = subresultant.next();
    assert_eq!(Polynomial::new(vec![224167500, -169966350]), subresultant.poly1);
    // The remaining steps cannot work because the coefficients break u32
    // (-117x^2 + -225x^1 + 441x^0) * -169966350
    // subresultant = subresultant.next();
    // assert_eq!(Polynomial::new(vec![1]), subresultant.poly1);
    // subresultant = subresultant.next();
    // assert!(subresultant.is_trivial());

    // Primitive subresultants
    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(Polynomial::new(vec![21, -9, -4, 0, 5, 0, 3]), subresultant.poly1);
    subresultant = subresultant.next_primitive();
    assert_eq!(Polynomial::new(vec![3, 0, -1, 0, 5]), subresultant.poly1);
    subresultant = subresultant.next_primitive();
    assert_eq!(Polynomial::new(vec![-49, 25, 13]), subresultant.poly1);
    subresultant = subresultant.next_primitive();
    assert_eq!(Polynomial::new(vec![-6150, 4663]), subresultant.poly1);
    subresultant = subresultant.next_primitive();
    assert_eq!(Polynomial::new(vec![1]), subresultant.poly1);
    subresultant = subresultant.next_primitive();
    assert!(subresultant.is_trivial());

    // Primitive gcd
    let subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(Polynomial::new(vec![1]), subresultant.into_primitive_gcd());

    // (x - 1)^2 (x - 26)^2 = (x^2 - 27 x + 26)^2 = x^4 - 54 x^3 + 781 x^2 - 1404 x + 676
    let a = Polynomial::new_big(vec![676, -1404, 781, -54, 1]);
    // f' = 4 x^3 - 162 x^2 + 1562 x - 1404
    let b = Polynomial::new_big(vec![-1404, 1562, -162, 4]);
    let mut subresultant = PolynomialSubresultant::new(a.clone(), b.clone());
    assert_eq!(Polynomial::new_big(vec![-65000, 67500, -2500]), subresultant.clone().into_gcd());
    assert_eq!(Polynomial::new_big(vec![-26, 27, -1]), subresultant.clone().into_factored_gcd());
    assert_eq!(Polynomial::new_big(vec![26, -27, 1]), subresultant.clone().into_primitive_gcd());
    assert_eq!(Polynomial::new_big(vec![-1404, 1562, -162, 4]), subresultant.poly1);
    subresultant = subresultant.next();
    assert_eq!(Polynomial::new_big(vec![-65000, 67500, -2500]), subresultant.poly1);
    subresultant = subresultant.next();
    assert!(subresultant.is_trivial());

}
