//! Base analyzer and multimodular basis provisioner.
//!
//! The original module tried to "activate only the primes dividing the natural
//! base" and leave the rest idle to save power. That is **mathematically
//! unsound**: CRT reconstruction needs *all* channels, so dropping channels
//! makes the result recoverable only modulo a tiny number — useless.
//!
//! This reframed dispatcher does the two jobs that are actually well-posed:
//!
//! 1. **Exactness / base analysis.** Report the natural base (LCM of the radicals
//!    of the operands' denominators) and whether a result is exact in a base.
//!    This *classifies* the answer; it never skips arithmetic.
//! 2. **Basis provisioning.** Estimate the result's bit-height and grow the
//!    [`Basis`] so the multimodular pipeline has enough primes to CRT +
//!    rational-reconstruct without aliasing.
//!
//! It also exposes the **per-value channel validity mask**: the channels whose
//! prime divides a value's denominator are non-integral at that prime and are
//! excluded from *that value's* reconstruction. This is correctness bookkeeping
//! for one number — never a global optimization.

use num_bigint::BigInt;
use num_traits::Zero;

use crate::basis::Basis;
use crate::primes::lcm;
use crate::rational::RnsRational;

/// A plan describing the base structure and basis requirements of an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchPlan {
    /// LCM of the operands' natural bases (the minimal exact base).
    pub natural_base: u64,
    /// A-priori bound on the result's bit-height (drives the basis size).
    pub required_bits: u64,
    /// Bits of range the current basis actually provides.
    pub provisioned_bits: u64,
}

impl DispatchPlan {
    /// Whether the current basis is large enough for the estimated result.
    pub fn fits(&self) -> bool {
        self.provisioned_bits >= self.required_bits
    }

    /// Bound-tightness diagnostic in `[0, ∞)`: `required / provisioned`. Values
    /// `> 1` mean the basis must grow; values `≪ 1` mean it is oversized.
    pub fn tightness(&self) -> f64 {
        if self.provisioned_bits == 0 {
            f64::INFINITY
        } else {
            self.required_bits as f64 / self.provisioned_bits as f64
        }
    }
}

/// Analyzes operands and provisions the adaptive basis.
pub struct Dispatcher {
    basis: Basis,
}

impl Dispatcher {
    pub fn new(basis: Basis) -> Self {
        Dispatcher { basis }
    }

    /// The basis this dispatcher currently manages.
    pub fn basis(&self) -> &Basis {
        &self.basis
    }

    /// Reconstruction needs `M > 2·max(|p|, |q|)²`, i.e. `~2·max_bits + 2` bits.
    fn reconstruct_bits(p: &BigInt, q: &BigInt) -> u64 {
        let max_bits = p.bits().max(q.bits());
        2 * max_bits + 2
    }

    fn plan(&self, base: u64, p: &BigInt, q: &BigInt) -> DispatchPlan {
        DispatchPlan {
            natural_base: base,
            required_bits: Self::reconstruct_bits(p, q),
            provisioned_bits: self.basis.capacity_bits(),
        }
    }

    /// Plan an addition: `(p1·q2 + p2·q1) / (q1·q2)`.
    pub fn plan_add(&self, a: &RnsRational, b: &RnsRational) -> DispatchPlan {
        let (p1, q1) = a.to_pair();
        let (p2, q2) = b.to_pair();
        let p = &p1 * &q2 + &p2 * &q1;
        let q = &q1 * &q2;
        self.plan(lcm(a.natural_base(), b.natural_base()), &p, &q)
    }

    /// Plan a multiplication: `(p1·p2) / (q1·q2)`.
    pub fn plan_mul(&self, a: &RnsRational, b: &RnsRational) -> DispatchPlan {
        let (p1, q1) = a.to_pair();
        let (p2, q2) = b.to_pair();
        let p = &p1 * &p2;
        let q = &q1 * &q2;
        self.plan(lcm(a.natural_base(), b.natural_base()), &p, &q)
    }

    /// Grow the basis so the (planned) result reconstructs without aliasing.
    pub fn provision(&mut self, plan: &DispatchPlan) {
        if !plan.fits() {
            self.basis = self.basis.extend_to_bits(plan.required_bits + 2);
        }
    }

    /// Execute an addition (exact regardless of the plan).
    pub fn execute_add(&self, a: &RnsRational, b: &RnsRational) -> RnsRational {
        a.add(b)
    }

    /// Execute a multiplication (exact regardless of the plan).
    pub fn execute_mul(&self, a: &RnsRational, b: &RnsRational) -> RnsRational {
        a.mul(b)
    }

    /// Per-value channel validity mask: indices of channels whose prime divides
    /// `denom` (and are therefore excluded from *that value's* reconstruction).
    pub fn invalid_channels(&self, denom: &BigInt) -> Vec<usize> {
        (0..self.basis.len())
            .filter(|&c| {
                let m = BigInt::from(self.basis.modulus(c));
                (denom % &m).is_zero()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b() -> Basis {
        Basis::standard()
    }

    #[test]
    fn natural_base_is_reported() {
        let d = Dispatcher::new(b());
        let sixth = RnsRational::from_fraction(1, 6, b());
        let quarter = RnsRational::from_fraction(1, 4, b());
        let plan = d.plan_add(&sixth, &quarter);
        // radical(6)=6, radical(4)=2, lcm=6
        assert_eq!(plan.natural_base, 6);
    }

    #[test]
    fn integers_need_minimal_range() {
        let d = Dispatcher::new(b());
        let a = RnsRational::from_int(3, b());
        let c = RnsRational::from_int(5, b());
        let plan = d.plan_add(&a, &c);
        assert_eq!(plan.natural_base, 1);
        assert!(plan.fits()); // standard basis dwarfs an 8-bit result
        assert!(plan.tightness() < 1.0);
    }

    #[test]
    fn small_basis_triggers_provisioning() {
        let mut d = Dispatcher::new(Basis::with_bits(40));
        // a large rational whose reconstruction needs > 40 bits
        let big: BigInt = BigInt::from(1u64) << 100;
        let a = RnsRational::new(big.clone(), BigInt::from(1), Basis::with_bits(40));
        let c = RnsRational::from_int(1, Basis::with_bits(40));
        let plan = d.plan_add(&a, &c);
        assert!(!plan.fits());
        d.provision(&plan);
        assert!(d.basis().capacity_bits() >= plan.required_bits);
    }

    #[test]
    fn execution_is_exact() {
        let d = Dispatcher::new(b());
        let sixth = RnsRational::from_fraction(1, 6, b());
        let quarter = RnsRational::from_fraction(1, 4, b());
        assert_eq!(
            d.execute_add(&sixth, &quarter),
            RnsRational::from_fraction(5, 12, b())
        );
    }
}
