#![allow(dead_code)]

use num::Zero;
use crate::{
    local_num::{LocalInteger, LocalOne},
    normed::Normed,
};
use super::Polynomial;


// Euclidean division with pseudo-remainder sequences
//   See https://en.wikipedia.org/wiki/Polynomial_greatest_common_divisor#Pseudo-remainder_sequences
//   and https://pqnelson.github.io/org-notes/math/rings/polynomial.html
//   and https://people.eecs.berkeley.edu/~fateman/282/F%20Wright%20notes/week5.pdf
//
// Note: as is, this method creates large coefficients; consider different strategies, e.g. primitive or subresultant

// input: a and b, univariate polynomials, nonzero
// output: q (pseudo-quotient pquo), r (pseudo-remainder prem), and C
//   q(x), r(x), and C such that C a(x) = q(x) b(x) + r(x)
//
// begin
//     da = deg(a)
//     db = deg(b)
//     lb = leading_coefficient(b)
//     n = 0
//     q = 0
//     r = a
//     while let dr = deg(r) >= d
//         lr = leading_coefficient(r)
//         s = lr x^(dr-db)
//         n += 1
//         q = lb q + s
//         r = lb r − s b
//     (q, r, lb^n)
// end

pub fn euclidean_pseudo_div<T>(poly0: Polynomial<T>, poly1: &Polynomial<T>) -> (Polynomial<T>, Polynomial<T>, T)
where T: Clone + LocalInteger {

    let mut q = Polynomial::zero();
    let (db, lb) = match (poly0.degree_and_leading_coefficient(), poly1.degree_and_leading_coefficient()) {
        (Some(_), Some((db, lb))) => (db, lb),
        (None, Some((_, lb))) => {
            return (q, poly0, lb.local_one());
        },
        (Some((_, la)), None) => {
            return (q, poly0, la.local_zero());
        },
        (None, None) => {
            panic!("Cannot perform pseudo-division if both inputs are zero");
        }
    };
    let mut r = poly0;

    let mut lb_to_n = lb.local_one();
    while let Some((dr, lr)) = r.degree_and_leading_coefficient().filter(|(dr, _)| *dr >= db) {
        let s = Polynomial::monomial(dr - db, lr.clone());
        lb_to_n = lb_to_n * lb.clone();
        q = q.mul_coefficients(lb.clone()) + s.clone();
        r = r.mul_coefficients(lb.clone()) - s * poly1.clone();
    }

    (q, r, lb_to_n)

}

// Euclid's GCD with pseudo-remainder sequences
pub fn pseudo_rem_gcd<T>(a: Polynomial<T>, b: Polynomial<T>) -> Polynomial<T>
where T: Clone + LocalInteger {
    let subresultant = PolynomialSubresultant::new(a, b);
    subresultant.into_gcd()
}

// GCD with pseudo-remainder sequences, taking out the GCD of the factors but NOT the leading unit
pub fn factored_pseudo_rem_gcd<T>(a: Polynomial<T>, b: Polynomial<T>) -> Polynomial<T>
where T: Clone + LocalInteger {
    let subresultant = PolynomialSubresultant::new(a, b);
    subresultant.into_factored_gcd()
}

// GCD with pseudo-remainder sequences, taking out the GCD of the factors AND the leading unit
pub fn primitive_pseudo_rem_gcd<T>(a: Polynomial<T>, b: Polynomial<T>) -> Polynomial<T>
where T: Clone + LocalInteger + Normed, T::Unit: LocalOne {
    let subresultant = PolynomialSubresultant::new(a, b);
    subresultant.into_primitive_gcd()
}


#[derive(Debug, Clone)]
pub (super) struct PolynomialSubresultant<T>
where T: Clone + LocalInteger {
    pub (super) poly0: Polynomial<T>,
    pub poly1: Polynomial<T>,
}

impl<T> PolynomialSubresultant<T>
where T: Clone + LocalInteger {

    pub (super) fn new(poly0: Polynomial<T>, poly1: Polynomial<T>) -> Self {
        Self {
            poly0,
            poly1,
        }
    }

    pub (super) fn into_gcd(self) -> Polynomial<T> {
        let mut s = self;
        while !s.is_trivial() {
            s = s.next();
        }
        s.poly0
    }

    pub (super) fn into_factored_gcd(self) -> Polynomial<T> {
        let mut s = self;
        s.poly0 = s.poly0.factor_coefficients();
        s.poly1 = s.poly1.factor_coefficients();
        while !s.is_trivial() {
            s = s.next_factored();
        }
        s.poly0
    }

    pub (super) fn into_primitive_gcd(self) -> Polynomial<T>
    where T: Normed, T::Unit: LocalOne {
        let mut s = self;
        s.poly0 = s.poly0.primitive_monic();
        s.poly1 = s.poly1.primitive_monic();
        while !s.is_trivial() {
            s = s.next_primitive();
        }
        s.poly0
    }

    pub (super) fn is_trivial(&self) -> bool {
        self.poly1.is_zero()
    }

    pub (super) fn next_factored(self) -> Self {
        let mut s = self.next();
        s.poly1 = s.poly1.factor_coefficients();
        s
    }

    pub (super) fn next_primitive(self) -> Self
    where T: Normed, T::Unit: LocalOne {
        let mut s = self.next();
        s.poly1 = s.poly1.primitive_monic();
        s
    }

    pub (super) fn next(self) -> Self {

        let Some(d0) = self.poly0.degree() else {
            return Self {
                poly0: self.poly1,
                poly1: Polynomial::zero(),
            }
        };

        let Some(d1) = self.poly1.degree() else {
            return Self {
                poly0: Polynomial::zero(),
                poly1: Polynomial::zero(),
            };
        };

        if d0 < d1 {
            Self {
                poly0: self.poly1,
                poly1: self.poly0,
            }
        } else {
            let (_quotient, remainder, _mult) = euclidean_pseudo_div(self.poly0.clone(), &self.poly1);
            Self {
                poly0: self.poly1,
                poly1: remainder,
            }
        }

    }

}
