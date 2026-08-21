//! Level 4 — 𝒮. Symbolic expression trees and an identity graph.
//!
//! The point of this level is that most computations involving π, e, √2, … never
//! need decimal digits — they need *algebraic relationships*. `simplify(sin(π))`
//! returns `Integer(0)` by a table lookup, in O(1), rather than evaluating
//! `sin(3.14159…) ≈ 1.2e-16` the way floating point does.

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};

/// A symbolic numeric expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolicExpr {
    Integer(BigInt),
    /// `p/q` (not necessarily reduced until simplified).
    Rational(BigInt, BigInt),
    /// `√n` with a simplified (square-free) radicand.
    Sqrt { radicand: BigInt },
    /// `(p/q)·√n`.
    ScaledSqrt { coeff: (BigInt, BigInt), rad: BigInt },
    Pi,
    E,
    Add(Vec<SymbolicExpr>),
    Mul(Vec<SymbolicExpr>),
    Pow { base: Box<SymbolicExpr>, exp: Box<SymbolicExpr> },
    Sin(Box<SymbolicExpr>),
    Cos(Box<SymbolicExpr>),
    Exp(Box<SymbolicExpr>),
    Ln(Box<SymbolicExpr>),
}

use SymbolicExpr::*;

impl SymbolicExpr {
    pub fn int(n: i64) -> Self {
        Integer(BigInt::from(n))
    }
    pub fn rational(p: i64, q: i64) -> Self {
        Rational(BigInt::from(p), BigInt::from(q))
    }
    pub fn sqrt(n: i64) -> Self {
        Sqrt { radicand: BigInt::from(n) }
    }
    pub fn add(terms: Vec<SymbolicExpr>) -> Self {
        Add(terms)
    }
    pub fn mul(factors: Vec<SymbolicExpr>) -> Self {
        Mul(factors)
    }
    pub fn sin(x: SymbolicExpr) -> Self {
        Sin(Box::new(x))
    }
    pub fn cos(x: SymbolicExpr) -> Self {
        Cos(Box::new(x))
    }
    pub fn exp(x: SymbolicExpr) -> Self {
        Exp(Box::new(x))
    }
    pub fn ln(x: SymbolicExpr) -> Self {
        Ln(Box::new(x))
    }

    /// Rational value of this node if it is a numeric constant.
    fn as_rational(&self) -> Option<(BigInt, BigInt)> {
        match self {
            Integer(n) => Some((n.clone(), BigInt::one())),
            Rational(p, q) => Some((p.clone(), q.clone())),
            _ => None,
        }
    }
}

/// Tower classification of a symbolic expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TowerLevel {
    Integer,
    Rational,
    Algebraic,
    Symbolic,
    Transcendental,
}

/// Classify an expression by the lowest tower level that can hold it.
pub fn tower_level(expr: &SymbolicExpr) -> TowerLevel {
    match expr {
        Integer(_) => TowerLevel::Integer,
        Rational(_, _) => TowerLevel::Rational,
        Sqrt { .. } | ScaledSqrt { .. } => TowerLevel::Algebraic,
        Pi | E | Sin(_) | Cos(_) | Exp(_) | Ln(_) => TowerLevel::Transcendental,
        Add(t) => t.iter().map(tower_level).max_by_key(level_rank).unwrap_or(TowerLevel::Integer),
        Mul(t) => t.iter().map(tower_level).max_by_key(level_rank).unwrap_or(TowerLevel::Integer),
        Pow { base, .. } => tower_level(base).max_symbolic(),
    }
}

fn level_rank(l: &TowerLevel) -> u8 {
    match l {
        TowerLevel::Integer => 0,
        TowerLevel::Rational => 1,
        TowerLevel::Algebraic => 2,
        TowerLevel::Symbolic => 3,
        TowerLevel::Transcendental => 4,
    }
}

impl TowerLevel {
    fn max_symbolic(self) -> TowerLevel {
        if level_rank(&self) >= level_rank(&TowerLevel::Symbolic) {
            self
        } else {
            TowerLevel::Symbolic
        }
    }
}

/// Holds the rewrite rules. For v0.1 the rules are encoded directly in
/// [`IdentityGraph::simplify`]; the type exists so the API can grow into a real
/// rule database later.
pub struct IdentityGraph;

impl Default for IdentityGraph {
    fn default() -> Self {
        Self::standard()
    }
}

impl IdentityGraph {
    /// The standard set of algebraic, trigonometric, and exponential identities.
    pub fn standard() -> Self {
        IdentityGraph
    }

    /// Simplify an expression to a fixed point.
    pub fn simplify(&self, expr: SymbolicExpr) -> SymbolicExpr {
        let mut current = expr;
        for _ in 0..64 {
            let next = self.step(current.clone());
            if next == current {
                return next;
            }
            current = next;
        }
        current
    }

    /// One bottom-up simplification pass.
    fn step(&self, expr: SymbolicExpr) -> SymbolicExpr {
        match expr {
            Add(terms) => self.simplify_add(terms),
            Mul(factors) => self.simplify_mul(factors),
            Sin(x) => self.simplify_sin(self.step(*x)),
            Cos(x) => self.simplify_cos(self.step(*x)),
            Exp(x) => self.simplify_exp(self.step(*x)),
            Ln(x) => self.simplify_ln(self.step(*x)),
            Pow { base, exp } => Pow {
                base: Box::new(self.step(*base)),
                exp: Box::new(self.step(*exp)),
            },
            Rational(p, q) => normalize_rational(p, q),
            other => other,
        }
    }

    fn simplify_add(&self, terms: Vec<SymbolicExpr>) -> SymbolicExpr {
        let mut const_num = BigInt::zero();
        let mut const_den = BigInt::one();
        let mut others: Vec<SymbolicExpr> = Vec::new();
        for t in terms {
            let t = self.step(t);
            match &t {
                Add(inner) => {
                    // Flatten nested sums.
                    for it in inner.clone() {
                        self.accumulate_add(it, &mut const_num, &mut const_den, &mut others);
                    }
                }
                _ => self.accumulate_add(t, &mut const_num, &mut const_den, &mut others),
            }
        }
        let mut result: Vec<SymbolicExpr> = Vec::new();
        if !const_num.is_zero() {
            result.push(normalize_rational(const_num, const_den));
        }
        result.append(&mut others);
        match result.len() {
            0 => SymbolicExpr::int(0),
            1 => result.into_iter().next().unwrap(),
            _ => Add(result),
        }
    }

    fn accumulate_add(
        &self,
        t: SymbolicExpr,
        num: &mut BigInt,
        den: &mut BigInt,
        others: &mut Vec<SymbolicExpr>,
    ) {
        if let Some((p, q)) = t.as_rational() {
            // num/den + p/q
            *num = &*num * &q + &p * &*den;
            *den = &*den * &q;
        } else {
            others.push(t);
        }
    }

    fn simplify_mul(&self, factors: Vec<SymbolicExpr>) -> SymbolicExpr {
        let mut coeff_num = BigInt::one();
        let mut coeff_den = BigInt::one();
        let mut radicand = BigInt::one();
        let mut others: Vec<SymbolicExpr> = Vec::new();
        let mut is_zero = false;

        let mut stack: Vec<SymbolicExpr> = factors.into_iter().map(|f| self.step(f)).collect();
        while let Some(f) = stack.pop() {
            match f {
                Mul(inner) => stack.extend(inner.into_iter().map(|f| self.step(f))),
                Integer(n) => {
                    if n.is_zero() {
                        is_zero = true;
                    }
                    coeff_num *= n;
                }
                Rational(p, q) => {
                    if p.is_zero() {
                        is_zero = true;
                    }
                    coeff_num *= p;
                    coeff_den *= q;
                }
                Sqrt { radicand: r } => radicand *= r,
                ScaledSqrt { coeff: (a, b), rad } => {
                    coeff_num *= a;
                    coeff_den *= b;
                    radicand *= rad;
                }
                other => others.push(other),
            }
        }

        if is_zero {
            return SymbolicExpr::int(0);
        }

        // Fold the combined radical.
        if !radicand.is_one() {
            match simplify_sqrt(radicand) {
                Integer(k) => coeff_num *= k,
                ScaledSqrt { coeff: (a, b), rad } => {
                    coeff_num *= a;
                    coeff_den *= b;
                    others.push(Sqrt { radicand: rad });
                }
                Sqrt { radicand: r } => others.push(Sqrt { radicand: r }),
                e => others.push(e),
            }
        }

        // Reduce the rational coefficient.
        let g = coeff_num.gcd(&coeff_den);
        if !g.is_zero() {
            coeff_num /= &g;
            coeff_den /= &g;
        }
        if coeff_den.is_negative() {
            coeff_num = -coeff_num;
            coeff_den = -coeff_den;
        }

        let coeff_is_one = coeff_num.is_one() && coeff_den.is_one();

        // Special-case a single radical times a rational coefficient.
        if others.len() == 1 {
            if let Sqrt { radicand: r } = &others[0] {
                if coeff_is_one {
                    return Sqrt { radicand: r.clone() };
                }
                return ScaledSqrt {
                    coeff: (coeff_num, coeff_den),
                    rad: r.clone(),
                };
            }
        }

        let mut result: Vec<SymbolicExpr> = Vec::new();
        if !coeff_is_one {
            result.push(normalize_rational(coeff_num, coeff_den));
        }
        result.append(&mut others);
        match result.len() {
            0 => SymbolicExpr::int(1),
            1 => result.into_iter().next().unwrap(),
            _ => Mul(result),
        }
    }

    fn simplify_sin(&self, x: SymbolicExpr) -> SymbolicExpr {
        if let Integer(n) = &x {
            if n.is_zero() {
                return SymbolicExpr::int(0);
            }
        }
        if let Some((a, b)) = as_pi_multiple(&x) {
            let (a, b) = reduce(a, b);
            // sin(k·π) for k ∈ {1/6, 1/4, 1/3, 1/2, 1}.
            if let Some(v) = sin_pi_table(&a, &b) {
                return v;
            }
        }
        Sin(Box::new(x))
    }

    fn simplify_cos(&self, x: SymbolicExpr) -> SymbolicExpr {
        if let Integer(n) = &x {
            if n.is_zero() {
                return SymbolicExpr::int(1);
            }
        }
        if let Some((a, b)) = as_pi_multiple(&x) {
            let (a, b) = reduce(a, b);
            if let Some(v) = cos_pi_table(&a, &b) {
                return v;
            }
        }
        Cos(Box::new(x))
    }

    fn simplify_exp(&self, x: SymbolicExpr) -> SymbolicExpr {
        if let Integer(n) = &x {
            if n.is_zero() {
                return SymbolicExpr::int(1);
            }
        }
        Exp(Box::new(x))
    }

    fn simplify_ln(&self, x: SymbolicExpr) -> SymbolicExpr {
        if let Integer(n) = &x {
            if n.is_one() {
                return SymbolicExpr::int(0);
            }
        }
        Ln(Box::new(x))
    }
}

/// Normalize `p/q` into the simplest `Integer`/`Rational` form.
fn normalize_rational(mut p: BigInt, mut q: BigInt) -> SymbolicExpr {
    if q.is_zero() {
        return Rational(p, q); // degenerate; leave as-is
    }
    if q.is_negative() {
        p = -p;
        q = -q;
    }
    let g = p.gcd(&q);
    if !g.is_zero() {
        p /= &g;
        q /= &g;
    }
    if q.is_one() {
        Integer(p)
    } else {
        Rational(p, q)
    }
}

/// Reduce a fraction `(a, b)` (b kept positive).
fn reduce(mut a: BigInt, mut b: BigInt) -> (BigInt, BigInt) {
    if b.is_negative() {
        a = -a;
        b = -b;
    }
    let g = a.gcd(&b);
    if !g.is_zero() {
        a /= &g;
        b /= &g;
    }
    (a, b)
}

/// Simplify `√n`: pull out the largest square factor.
fn simplify_sqrt(n: BigInt) -> SymbolicExpr {
    if n.is_negative() || n.is_zero() {
        return Sqrt { radicand: n };
    }
    let nu = match n.to_u128() {
        Some(v) => v,
        None => return Sqrt { radicand: n },
    };
    let mut square = 1u128;
    let mut rad = nu;
    let mut d = 2u128;
    while d * d <= rad {
        while rad % (d * d) == 0 {
            rad /= d * d;
            square *= d;
        }
        d += 1;
    }
    let s = BigInt::from(square);
    let r = BigInt::from(rad);
    if rad == 1 {
        Integer(s)
    } else if square == 1 {
        Sqrt { radicand: r }
    } else {
        ScaledSqrt { coeff: (s, BigInt::one()), rad: r }
    }
}

/// If `expr == k·π` for a rational `k`, return `k` as `(num, den)`.
fn as_pi_multiple(expr: &SymbolicExpr) -> Option<(BigInt, BigInt)> {
    match expr {
        Pi => Some((BigInt::one(), BigInt::one())),
        Mul(factors) => {
            let mut num = BigInt::one();
            let mut den = BigInt::one();
            let mut pi_count = 0;
            for f in factors {
                match f {
                    Pi => pi_count += 1,
                    Integer(n) => num *= n,
                    Rational(p, q) => {
                        num *= p;
                        den *= q;
                    }
                    _ => return None,
                }
            }
            if pi_count == 1 {
                Some((num, den))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn frac_is(a: &BigInt, b: &BigInt, n: i64, d: i64) -> bool {
    *a == BigInt::from(n) && *b == BigInt::from(d)
}

/// `sin(a/b · π)` for the special angles.
fn sin_pi_table(a: &BigInt, b: &BigInt) -> Option<SymbolicExpr> {
    if a.is_zero() {
        return Some(SymbolicExpr::int(0));
    }
    if frac_is(a, b, 1, 1) {
        return Some(SymbolicExpr::int(0)); // sin(π) = 0
    }
    if frac_is(a, b, 1, 6) {
        return Some(SymbolicExpr::rational(1, 2));
    }
    if frac_is(a, b, 1, 4) {
        return Some(ScaledSqrt { coeff: (BigInt::one(), BigInt::from(2)), rad: BigInt::from(2) });
    }
    if frac_is(a, b, 1, 3) {
        return Some(ScaledSqrt { coeff: (BigInt::one(), BigInt::from(2)), rad: BigInt::from(3) });
    }
    if frac_is(a, b, 1, 2) {
        return Some(SymbolicExpr::int(1)); // sin(π/2) = 1
    }
    None
}

/// `cos(a/b · π)` for the special angles.
fn cos_pi_table(a: &BigInt, b: &BigInt) -> Option<SymbolicExpr> {
    if a.is_zero() {
        return Some(SymbolicExpr::int(1));
    }
    if frac_is(a, b, 1, 1) {
        return Some(SymbolicExpr::int(-1)); // cos(π) = -1
    }
    if frac_is(a, b, 1, 2) {
        return Some(SymbolicExpr::int(0)); // cos(π/2) = 0
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g() -> IdentityGraph {
        IdentityGraph::standard()
    }

    #[test]
    fn sin_pi_is_zero() {
        assert_eq!(g().simplify(SymbolicExpr::sin(Pi)), SymbolicExpr::int(0));
    }

    #[test]
    fn cos_pi_is_minus_one() {
        assert_eq!(g().simplify(SymbolicExpr::cos(Pi)), SymbolicExpr::int(-1));
    }

    #[test]
    fn sin_pi_over_six() {
        let expr = SymbolicExpr::sin(Mul(vec![SymbolicExpr::rational(1, 6), Pi]));
        assert_eq!(g().simplify(expr), SymbolicExpr::rational(1, 2));
    }

    #[test]
    fn exp_zero_is_one() {
        assert_eq!(g().simplify(SymbolicExpr::exp(SymbolicExpr::int(0))), SymbolicExpr::int(1));
    }

    #[test]
    fn ln_one_is_zero() {
        assert_eq!(g().simplify(SymbolicExpr::ln(SymbolicExpr::int(1))), SymbolicExpr::int(0));
    }

    #[test]
    fn sqrt_times_sqrt() {
        let expr = Mul(vec![SymbolicExpr::sqrt(2), SymbolicExpr::sqrt(2)]);
        assert_eq!(g().simplify(expr), SymbolicExpr::int(2));
    }

    #[test]
    fn x_times_zero() {
        let expr = Mul(vec![Pi, SymbolicExpr::int(0)]);
        assert_eq!(g().simplify(expr), SymbolicExpr::int(0));
    }

    #[test]
    fn add_zero_identity() {
        let expr = Add(vec![Pi, SymbolicExpr::int(0)]);
        assert_eq!(g().simplify(expr), Pi);
    }

    #[test]
    fn mul_one_identity() {
        let expr = Mul(vec![Pi, SymbolicExpr::int(1)]);
        assert_eq!(g().simplify(expr), Pi);
    }

    #[test]
    fn sqrt_eight_simplifies() {
        // √8 = 2√2
        assert_eq!(
            simplify_sqrt(BigInt::from(8)),
            ScaledSqrt { coeff: (BigInt::from(2), BigInt::one()), rad: BigInt::from(2) }
        );
    }

    #[test]
    fn classification() {
        assert_eq!(tower_level(&SymbolicExpr::int(3)), TowerLevel::Integer);
        assert_eq!(tower_level(&SymbolicExpr::sqrt(2)), TowerLevel::Algebraic);
        assert_eq!(tower_level(&Pi), TowerLevel::Transcendental);
    }
}
