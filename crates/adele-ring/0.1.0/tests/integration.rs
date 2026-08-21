//! Crate-level integration tests spanning multiple tower levels and both backends.

use adele_ring::backend::ArithmeticBackend;
use adele_ring::{
    cpu::CpuBackend, gpu::GpuBackend, AlgebraicNumber, Channels, ComputableReal, Dispatcher,
    IdentityGraph, Polynomial, RnsBatch, RnsInt, RnsRational, SymbolicExpr, TowerLevel, TowerValue,
};
use adele_ring::rns::garner_crt;

fn ch() -> Channels {
    Channels::standard(32)
}

// ── Backend correctness: CPU and GPU must match exactly ──────────────────────

#[test]
fn cpu_gpu_bit_identical() {
    let channels = ch();
    let a = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, channels.clone()); 256]);
    let b = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, channels.clone()); 256]);

    let cpu = CpuBackend::new();
    let cpu_add = cpu.batch_rns_add(&a, &b);
    let cpu_mul = cpu.batch_rns_mul(&a, &b);

    if let Ok(gpu) = GpuBackend::try_init() {
        assert_eq!(cpu_add.data, gpu.batch_rns_add(&a, &b).data);
        assert_eq!(cpu_mul.data, gpu.batch_rns_mul(&a, &b).data);
    }
}

// ── Level 0 & 1: integer and rational ────────────────────────────────────────

#[test]
fn rational_identities() {
    let f = |p, q| RnsRational::from_fraction(p, q, ch());
    assert_eq!(f(1, 6).add(&f(1, 10)).add(&f(1, 15)), f(1, 3));
    assert_eq!(f(1, 3).mul(&f(3, 1)), f(1, 1));
    assert_eq!(f(1, 10).add(&f(1, 5)), f(3, 10));
    assert_eq!(f(7, 8).sub(&f(3, 8)), f(1, 2));
}

#[test]
fn base_alignment() {
    let f = |p, q| RnsRational::from_fraction(p, q, ch());
    assert_eq!(f(1, 6).natural_base(), 6);
    assert_eq!(f(1, 12).natural_base(), 6);
    assert!(f(1, 6).exact_in_base(6));
    assert!(!f(1, 6).exact_in_base(10));
    assert!(f(1, 6).exact_in_base(30));
}

#[test]
fn garner_crt_examples() {
    assert_eq!(garner_crt(&[2, 3, 2], &[3, 5, 7]), num_bigint::BigUint::from(23u8));
    assert_eq!(garner_crt(&[0, 1, 0], &[2, 3, 5]), num_bigint::BigUint::from(10u8));
}

// ── Level 2: algebraic ───────────────────────────────────────────────────────

#[test]
fn algebraic_arithmetic() {
    let s2 = AlgebraicNumber::sqrt(2, ch());
    assert_eq!(s2.min_poly.degree(), 2);
    assert!(s2.to_rational().is_none());

    // √2 · √2 = 2  (degree-1 min poly => rational)
    assert_eq!(s2.mul(&s2).degree(), 1);

    // √2 · √3 = √6
    let s3 = AlgebraicNumber::sqrt(3, ch());
    assert_eq!(
        s2.mul(&s3).min_poly,
        Polynomial::from_int_coeffs(&[-6, 0, 1], ch()).monic()
    );

    // √2 + √2 = 2√2  (min poly x² - 8)
    assert_eq!(
        s2.add(&s2).min_poly,
        Polynomial::from_int_coeffs(&[-8, 0, 1], ch()).monic()
    );
}

#[test]
fn sturm_root_counts() {
    let p = Polynomial::from_int_coeffs(&[-2, 0, 1], ch()); // x² - 2
    let r = |a, b| {
        p.sturm_root_count(
            &RnsRational::from_int(a, ch()),
            &RnsRational::from_int(b, ch()),
        )
    };
    assert_eq!(r(-2, -1), 1);
    assert_eq!(r(1, 2), 1);
    assert_eq!(r(-2, 2), 2);
}

// ── Level 3: computable ──────────────────────────────────────────────────────

#[test]
fn computable_constants() {
    let pi = ComputableReal::pi(ch());
    assert!((pi.evaluate(10).to_f64() - std::f64::consts::PI).abs() < 1e-10);

    let e = ComputableReal::e(ch());
    assert!((e.evaluate(15).to_f64() - std::f64::consts::E).abs() < 1e-14);

    // precision contract: low-precision result within its stated bound
    let lo = pi.evaluate(5).to_f64();
    let hi = pi.evaluate(50).to_f64();
    assert!((lo - hi).abs() < 1e-5);

    // exact rational passes through unchanged at any precision
    let r = RnsRational::from_fraction(1, 3, ch());
    let cr = ComputableReal::from_rational(r.clone());
    assert_eq!(cr.evaluate(100), r);
}

// ── Level 4: symbolic identities ─────────────────────────────────────────────

#[test]
fn symbolic_identities() {
    let g = IdentityGraph::standard();
    assert_eq!(g.simplify(SymbolicExpr::sin(SymbolicExpr::Pi)), SymbolicExpr::int(0));
    assert_eq!(g.simplify(SymbolicExpr::cos(SymbolicExpr::Pi)), SymbolicExpr::int(-1));
    assert_eq!(
        g.simplify(SymbolicExpr::sin(SymbolicExpr::Mul(vec![
            SymbolicExpr::rational(1, 6),
            SymbolicExpr::Pi
        ]))),
        SymbolicExpr::rational(1, 2)
    );
    assert_eq!(g.simplify(SymbolicExpr::exp(SymbolicExpr::int(0))), SymbolicExpr::int(1));
    assert_eq!(
        g.simplify(SymbolicExpr::Mul(vec![SymbolicExpr::sqrt(2), SymbolicExpr::sqrt(2)])),
        SymbolicExpr::int(2)
    );
}

// ── Tower routing & dispatch ─────────────────────────────────────────────────

#[test]
fn tower_routing() {
    let r = RnsRational::from_fraction(2, 3, ch());
    assert_eq!(TowerValue::Rational(r).level(), TowerLevel::Rational);

    let r_int = RnsRational::from_fraction(6, 2, ch());
    assert_eq!(TowerValue::Rational(r_int).reduce().level(), TowerLevel::Integer);

    let s2 = TowerValue::Algebraic(AlgebraicNumber::sqrt(2, ch()));
    assert_eq!(s2.mul(&s2).reduce().level(), TowerLevel::Integer);
}

#[test]
fn dispatcher_efficiency() {
    let channels = Channels::standard(16);
    let dispatcher = Dispatcher::new(channels.clone());
    let sixth = RnsRational::from_fraction(1, 6, channels.clone());
    let quarter = RnsRational::from_fraction(1, 4, channels);
    let plan = dispatcher.plan_add(&sixth, &quarter);
    assert_eq!(plan.active_channels.len(), 2);
    assert!((dispatcher.channel_efficiency(&plan) - 2.0 / 16.0).abs() < 1e-12);
}
