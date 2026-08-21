use std::ops::{Add, Mul, Neg, Sub, Rem};
use itertools::{EitherOrBoth, Itertools, repeat_n};
use num::Zero;

use crate::{
    local_num::{LocalInteger, LocalOne, LocalZero},
    mapping::Differentiable,
};
use super::{euclid, Polynomial};


// === Polynomial ===

impl<T> Zero for Polynomial<T>
where T: Clone + LocalZero {
    fn zero() -> Self {
        Self::new(vec![])
    }
    fn is_zero(&self) -> bool {
        self.coefficients().all(T::is_local_zero)
    }
}


impl<T> Add for Polynomial<T>
where T: Clone + LocalZero {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {

        let new_coefficients = self
            .into_coefficients()
            .zip_longest(rhs.into_coefficients())
            .map(|new| match new {
                EitherOrBoth::Both(lhs, rhs ) => lhs + rhs,
                EitherOrBoth::Left(lhs) => lhs,
                EitherOrBoth::Right(rhs) => rhs,
            })
            .collect();

        Polynomial::new(new_coefficients)
    }
}

impl<T> Mul for Polynomial<T>
where T: Clone + LocalZero + LocalOne {
    type Output = Self;

    // Ignore suspicious + in the Mul impl
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Self) -> Self::Output {

        // NOTE: this could probably be made more efficient but works well as-is
        let mut out = Polynomial::new(vec![]);
        for (deg_lhs, lhs_term) in self.into_coefficients().enumerate() {
            let coefficients = repeat_n(lhs_term.local_zero(), deg_lhs)
                .chain(rhs.clone().into_coefficients())
                .map(|rhs_term| lhs_term.clone() * rhs_term)
                .collect();
            out = out + Polynomial::new(coefficients);
        }

        out
    }
}

impl<T> Neg for Polynomial<T>
where T: Clone + LocalZero + std::ops::Neg<Output=T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let coefficients = self.clone()
            .into_coefficients()
            .map(|a| -a)
            .collect();

        Polynomial::new(coefficients)
    }
}

impl<T> Sub for Polynomial<T>
where T: Clone + LocalZero + std::ops::Sub<Output=T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {

        let new_coefficients = self
            .into_coefficients()
            .zip_longest(rhs.into_coefficients())
            .map(|new| match new {
                EitherOrBoth::Both(lhs, rhs ) => lhs - rhs,
                EitherOrBoth::Left(lhs) => lhs,
                EitherOrBoth::Right(rhs) => rhs.local_zero() - rhs,
            })
            .collect();

        Polynomial::new(new_coefficients)
    }
}

impl<T> Rem for Polynomial<T>
where T: Clone + LocalInteger {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        let (_quotient, remainder, mult) = euclid::euclidean_pseudo_div(self, &rhs);
        remainder.map_coefficients(|c| c / mult.clone())
    }
}

impl<T> Differentiable for Polynomial<T>
where T: Clone + LocalZero + LocalOne {
    type Output = Self;
    fn into_derivative(self) -> Self {

        let Some((zero, one)) = self.coefficients().next().map(|c| (c.local_zero(), c.local_one())) else {
            return Self::new(vec![])
        };

        let mut deg = zero;
        let mut new_coefficients = vec![];
        for coeff in self.into_coefficients().skip(1) {
            deg = deg + one.clone();
            new_coefficients.push(deg.clone() * coeff.clone());
        }

        Self::new(new_coefficients)

    }
}
