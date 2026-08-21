//! The number tower router. Every value carries a [`TowerLevel`]; operations try
//! to *stay at the lowest (cheapest) level* they can, dropping down when a result
//! simplifies (e.g. √2·√2 = 2) and rising only when forced.
//!
//! When a caller asks for digits, the request flows *down* the tower: a symbolic
//! expression is simplified, then (if still not exact) elevated to a computable
//! real that produces the requested precision.

use num_bigint::BigInt;

use crate::algebraic::AlgebraicNumber;
use crate::computable::ComputableReal;
use crate::primes::factorize;
use crate::rational::RnsRational;
use crate::rns::{Channels, RnsInt};
use crate::symbolic::{IdentityGraph, SymbolicExpr};

/// The levels of the number tower, ordered cheapest-first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TowerLevel {
    Integer = 0,
    Rational = 1,
    Algebraic = 2,
    Computable = 3,
    Symbolic = 4,
}

/// A value somewhere in the number tower.
#[derive(Clone)]
pub enum TowerValue {
    Integer(RnsInt),
    Rational(RnsRational),
    Algebraic(AlgebraicNumber),
    Computable(ComputableReal),
    Symbolic(SymbolicExpr),
}

impl TowerValue {
    /// The level this value currently sits at.
    pub fn level(&self) -> TowerLevel {
        match self {
            TowerValue::Integer(_) => TowerLevel::Integer,
            TowerValue::Rational(_) => TowerLevel::Rational,
            TowerValue::Algebraic(_) => TowerLevel::Algebraic,
            TowerValue::Computable(_) => TowerLevel::Computable,
            TowerValue::Symbolic(_) => TowerLevel::Symbolic,
        }
    }

    /// Best-effort channels for this value (symbolic falls back to the standard set).
    pub fn channels(&self) -> Channels {
        match self {
            TowerValue::Integer(i) => i.channels.clone(),
            TowerValue::Rational(r) => r.channels.clone(),
            TowerValue::Algebraic(a) => a.channels.clone(),
            TowerValue::Computable(c) => c.channels(),
            TowerValue::Symbolic(_) => Channels::standard(32),
        }
    }

    /// Reduce to the lowest valid level (applied to a fixed point).
    pub fn reduce(&self) -> TowerValue {
        let mut current = self.clone();
        loop {
            let next = current.reduce_once();
            if next.level() == current.level() {
                return next;
            }
            current = next;
        }
    }

    fn reduce_once(&self) -> TowerValue {
        match self {
            TowerValue::Algebraic(a) if a.degree() == 1 => {
                TowerValue::Rational(a.to_rational().unwrap())
            }
            TowerValue::Rational(r) if r.is_integer() => {
                let p = r.to_pair().0;
                TowerValue::Integer(RnsInt::from_bigint(&p, r.channels.clone()))
            }
            TowerValue::Symbolic(e) => {
                let simplified = IdentityGraph::standard().simplify(e.clone());
                match symbolic_to_value(&simplified, &self.channels()) {
                    Some(v) => v,
                    None => TowerValue::Symbolic(simplified),
                }
            }
            other => other.clone(),
        }
    }

    /// Elevate to at least `target` (for mixed-level operations). Symbolic is the
    /// top; numeric levels rise one step at a time.
    pub fn elevate_to(&self, target: TowerLevel) -> TowerValue {
        let mut current = self.clone();
        while current.level() < target {
            current = current.elevate_once();
        }
        current
    }

    fn elevate_once(&self) -> TowerValue {
        match self {
            TowerValue::Integer(i) => {
                TowerValue::Rational(RnsRational::new(i.to_bigint(), BigInt::from(1), i.channels.clone()))
            }
            TowerValue::Rational(r) => TowerValue::Algebraic(AlgebraicNumber::from_rational(r.clone())),
            TowerValue::Algebraic(a) => TowerValue::Computable(a.to_computable()),
            TowerValue::Computable(_) => self.clone(), // cannot rise to symbolic
            TowerValue::Symbolic(_) => self.clone(),
        }
    }

    /// Decimal approximation (levels 0–3; symbolic is simplified/evaluated first).
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            TowerValue::Integer(i) => {
                Some(RnsRational::new(i.to_bigint(), BigInt::from(1), i.channels.clone()).to_f64())
            }
            TowerValue::Rational(r) => Some(r.to_f64()),
            TowerValue::Algebraic(a) => Some(a.to_f64()),
            TowerValue::Computable(c) => Some(c.evaluate_f64()),
            TowerValue::Symbolic(_) => self.reduce().non_symbolic_to_f64(),
        }
    }

    fn non_symbolic_to_f64(&self) -> Option<f64> {
        match self {
            TowerValue::Symbolic(e) => {
                // Try to evaluate a transcendental constant numerically.
                let ch = self.channels();
                symbolic_to_computable(e, &ch).map(|c| c.evaluate_f64())
            }
            other => other.to_f64(),
        }
    }

    /// Produce exact digits to `precision` decimal places, as a rational.
    pub fn digits(&self, precision: u64) -> RnsRational {
        match self {
            TowerValue::Integer(i) => {
                RnsRational::new(i.to_bigint(), BigInt::from(1), i.channels.clone())
            }
            TowerValue::Rational(r) => r.clone(),
            TowerValue::Algebraic(a) => {
                let mut clone = a.clone();
                let target = RnsRational::new(
                    BigInt::from(1),
                    BigInt::from(10u8).pow((precision + 1) as u32),
                    a.channels.clone(),
                );
                clone.refine_interval(&target);
                clone.interval.0.midpoint(&clone.interval.1)
            }
            TowerValue::Computable(c) => c.evaluate(precision),
            TowerValue::Symbolic(e) => {
                let ch = self.channels();
                let simplified = IdentityGraph::standard().simplify(e.clone());
                if let Some(v) = symbolic_to_value(&simplified, &ch) {
                    return v.digits(precision);
                }
                match symbolic_to_computable(&simplified, &ch) {
                    Some(c) => c.evaluate(precision),
                    None => panic!("cannot produce digits for irreducible symbolic expression"),
                }
            }
        }
    }

    // ── Arithmetic ──────────────────────────────────────────────────────────

    /// Addition, dropping the result to its lowest valid level.
    pub fn add(&self, other: &TowerValue) -> TowerValue {
        self.binop(other, Op::Add).reduce()
    }

    /// Multiplication, dropping the result to its lowest valid level.
    pub fn mul(&self, other: &TowerValue) -> TowerValue {
        self.binop(other, Op::Mul).reduce()
    }

    fn binop(&self, other: &TowerValue, op: Op) -> TowerValue {
        let lvl = self.level().max(other.level());
        if lvl == TowerLevel::Symbolic {
            let a = self.to_symbolic();
            let b = other.to_symbolic();
            let expr = match op {
                Op::Add => SymbolicExpr::Add(vec![a, b]),
                Op::Mul => SymbolicExpr::Mul(vec![a, b]),
            };
            return TowerValue::Symbolic(IdentityGraph::standard().simplify(expr));
        }
        let a = self.elevate_to(lvl);
        let b = other.elevate_to(lvl);
        match (a, b) {
            (TowerValue::Integer(x), TowerValue::Integer(y)) => TowerValue::Integer(match op {
                Op::Add => x.add(&y),
                Op::Mul => x.mul(&y),
            }),
            (TowerValue::Rational(x), TowerValue::Rational(y)) => TowerValue::Rational(match op {
                Op::Add => x.add(&y),
                Op::Mul => x.mul(&y),
            }),
            (TowerValue::Algebraic(x), TowerValue::Algebraic(y)) => TowerValue::Algebraic(match op {
                Op::Add => x.add(&y),
                Op::Mul => x.mul(&y),
            }),
            (TowerValue::Computable(x), TowerValue::Computable(y)) => TowerValue::Computable(match op {
                Op::Add => x.add(&y),
                Op::Mul => x.mul(&y),
            }),
            _ => unreachable!("levels were equalized before the operation"),
        }
    }

    /// Square root: a perfect square drops to `Integer`, otherwise rises to
    /// `Algebraic`.
    pub fn sqrt(&self) -> TowerValue {
        let ch = self.channels();
        if let TowerValue::Integer(i) = self {
            let n = i.to_bigint();
            if let Some(nu) = bigint_to_u64(&n) {
                let root = (nu as f64).sqrt().round() as u64;
                if root * root == nu {
                    return TowerValue::Integer(RnsInt::from_bigint(&BigInt::from(root), ch));
                }
                return TowerValue::Algebraic(AlgebraicNumber::sqrt(nu, ch)).reduce();
            }
        }
        // General: take the f64 value and build an algebraic square root if integral.
        let v = self.to_f64().unwrap_or(f64::NAN);
        if v >= 0.0 && v.fract() == 0.0 {
            return TowerValue::Algebraic(AlgebraicNumber::sqrt(v as u64, ch)).reduce();
        }
        panic!("sqrt of non-integer values is not supported at the tower level yet");
    }

    /// sin: checks the symbolic identity graph; otherwise stays symbolic.
    pub fn sin(&self) -> TowerValue {
        let expr = SymbolicExpr::Sin(Box::new(self.to_symbolic()));
        TowerValue::Symbolic(IdentityGraph::standard().simplify(expr)).reduce()
    }

    fn to_symbolic(&self) -> SymbolicExpr {
        match self {
            TowerValue::Integer(i) => SymbolicExpr::Integer(i.to_bigint()),
            TowerValue::Rational(r) => {
                let (p, q) = r.to_pair();
                SymbolicExpr::Rational(p, q)
            }
            TowerValue::Symbolic(e) => e.clone(),
            // Algebraic/Computable have no faithful finite symbolic form here.
            TowerValue::Algebraic(a) => {
                if let Some(r) = a.to_rational() {
                    let (p, q) = r.to_pair();
                    SymbolicExpr::Rational(p, q)
                } else {
                    panic!("cannot lift this algebraic number to symbolic form")
                }
            }
            TowerValue::Computable(_) => panic!("cannot lift a computable real to symbolic form"),
        }
    }
}

#[derive(Clone, Copy)]
enum Op {
    Add,
    Mul,
}

fn bigint_to_u64(n: &BigInt) -> Option<u64> {
    use num_traits::ToPrimitive;
    n.to_u64()
}

/// Map a fully simplified symbolic expression onto a concrete tower value.
fn symbolic_to_value(e: &SymbolicExpr, ch: &Channels) -> Option<TowerValue> {
    match e {
        SymbolicExpr::Integer(n) => Some(TowerValue::Integer(RnsInt::from_bigint(n, ch.clone()))),
        SymbolicExpr::Rational(p, q) => {
            Some(TowerValue::Rational(RnsRational::new(p.clone(), q.clone(), ch.clone())))
        }
        SymbolicExpr::Sqrt { radicand } => {
            let nu = bigint_to_u64(radicand)?;
            Some(TowerValue::Algebraic(AlgebraicNumber::sqrt(nu, ch.clone())))
        }
        SymbolicExpr::ScaledSqrt { coeff: (a, b), rad } => {
            let nu = bigint_to_u64(rad)?;
            let s = AlgebraicNumber::sqrt(nu, ch.clone());
            let coeff = RnsRational::new(a.clone(), b.clone(), ch.clone());
            Some(TowerValue::Algebraic(s).mul(&TowerValue::Rational(coeff)))
        }
        _ => None,
    }
}

/// Build a computable real from a symbolic expression (used for digit requests
/// on transcendentals like π+1).
fn symbolic_to_computable(e: &SymbolicExpr, ch: &Channels) -> Option<ComputableReal> {
    match e {
        SymbolicExpr::Integer(n) => Some(ComputableReal::from_rational(RnsRational::new(
            n.clone(),
            BigInt::from(1),
            ch.clone(),
        ))),
        SymbolicExpr::Rational(p, q) => Some(ComputableReal::from_rational(RnsRational::new(
            p.clone(),
            q.clone(),
            ch.clone(),
        ))),
        SymbolicExpr::Pi => Some(ComputableReal::pi(ch.clone())),
        SymbolicExpr::E => Some(ComputableReal::e(ch.clone())),
        SymbolicExpr::Sqrt { radicand } => {
            let r = RnsRational::new(radicand.clone(), BigInt::from(1), ch.clone());
            Some(ComputableReal::sqrt(r))
        }
        SymbolicExpr::Add(terms) => {
            let mut acc: Option<ComputableReal> = None;
            for t in terms {
                let c = symbolic_to_computable(t, ch)?;
                acc = Some(match acc {
                    Some(a) => a.add(&c),
                    None => c,
                });
            }
            acc
        }
        SymbolicExpr::Mul(factors) => {
            let mut acc: Option<ComputableReal> = None;
            for f in factors {
                let c = symbolic_to_computable(f, ch)?;
                acc = Some(match acc {
                    Some(a) => a.mul(&c),
                    None => c,
                });
            }
            acc
        }
        _ => None,
    }
}

/// Square-free check helper exposed for completeness (used by examples/tests).
pub fn is_perfect_square(n: u64) -> bool {
    let r = (n as f64).sqrt().round() as u64;
    r * r == n
}

#[allow(dead_code)]
fn _uses_factorize() {
    let _ = factorize(12);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch() -> Channels {
        Channels::standard(32)
    }

    #[test]
    fn rational_level() {
        let v = TowerValue::Rational(RnsRational::from_fraction(2, 3, ch()));
        assert_eq!(v.level(), TowerLevel::Rational);
    }

    #[test]
    fn rational_with_denom_one_reduces_to_integer() {
        let v = TowerValue::Rational(RnsRational::from_fraction(6, 2, ch()));
        assert_eq!(v.reduce().level(), TowerLevel::Integer);
    }

    #[test]
    fn sqrt2_times_sqrt2_drops_to_integer() {
        let s = TowerValue::Algebraic(AlgebraicNumber::sqrt(2, ch()));
        let prod = s.mul(&s);
        assert_eq!(prod.level(), TowerLevel::Integer);
        assert_eq!(prod.to_f64().unwrap().round(), 2.0);
    }

    #[test]
    fn mixed_level_add() {
        // Integer 1 + Rational 1/2 = 3/2.
        let a = TowerValue::Integer(RnsInt::from_i64(1, ch()));
        let b = TowerValue::Rational(RnsRational::from_fraction(1, 2, ch()));
        let sum = a.add(&b);
        assert_eq!(sum.level(), TowerLevel::Rational);
        assert!((sum.to_f64().unwrap() - 1.5).abs() < 1e-12);
    }

    #[test]
    fn pi_plus_one_digits() {
        // π + 1 stays symbolic, then digits() elevates to computable.
        let pi = TowerValue::Symbolic(SymbolicExpr::Pi);
        let one = TowerValue::Integer(RnsInt::from_i64(1, ch()));
        let sum = pi.add(&one);
        let d = sum.digits(20);
        assert!((d.to_f64() - (std::f64::consts::PI + 1.0)).abs() < 1e-12);
    }

    #[test]
    fn sin_pi_is_zero() {
        let pi = TowerValue::Symbolic(SymbolicExpr::Pi);
        let s = pi.sin();
        assert_eq!(s.to_f64().unwrap(), 0.0);
    }
}
