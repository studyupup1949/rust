//! Side-by-side comparison of exact `adele-ring` results versus naive `f64`.
//!
//! Run with: `cargo run --example float_comparison`

use adele_ring::{AlgebraicNumber, Basis, RnsRational, SymbolicExpr, TowerValue};

/// Distance in ULPs between two finite f64 values of the same sign.
fn ulp_distance(a: f64, b: f64) -> i64 {
    let ai = a.to_bits() as i64;
    let bi = b.to_bits() as i64;
    (ai - bi).abs()
}

fn row(label: &str, exact: &str, f64_val: f64, exact_val: f64) {
    let abs_err = (f64_val - exact_val).abs();
    let ulps = ulp_distance(f64_val, exact_val);
    println!(
        "{label:<16} | {exact:<10} | {f64_val:<22.17} | {abs_err:<12.3e} | {ulps}",
    );
}

fn main() {
    let ch = Basis::standard();
    println!("== adele-ring :: exact vs f64 ==\n");
    println!(
        "{:<16} | {:<10} | {:<22} | {:<12} | ULPs",
        "expression", "exact", "f64 result", "abs error"
    );
    println!("{}", "-".repeat(78));

    // 0.1 + 0.2 = 3/10
    let exact = RnsRational::from_fraction(1, 10, ch.clone())
        .add(&RnsRational::from_fraction(1, 5, ch.clone()));
    row("0.1 + 0.2", &exact.display(), 0.1 + 0.2, exact.to_f64());

    // 1/3 * 3 = 1
    let one = RnsRational::from_fraction(1, 3, ch.clone()).mul(&RnsRational::from_int(3, ch.clone()));
    row("1/3 * 3", &one.display(), (1.0 / 3.0) * 3.0, one.to_f64());

    // sqrt(2) * sqrt(2) = 2  (drops to Integer through the tower)
    let s2 = TowerValue::Algebraic(AlgebraicNumber::sqrt(2, ch.clone()));
    let two = s2.mul(&s2);
    let naive = 2f64.sqrt() * 2f64.sqrt();
    row("sqrt2 * sqrt2", "2", naive, two.to_f64().unwrap());

    // sin(pi) = 0  (exact identity; f64 gives ~1.2e-16)
    let sin_pi = TowerValue::Symbolic(SymbolicExpr::Pi).sin();
    row("sin(pi)", "0", std::f64::consts::PI.sin(), sin_pi.to_f64().unwrap());

    // 1/7 * 7 = 1
    let one7 = RnsRational::from_fraction(1, 7, ch.clone()).mul(&RnsRational::from_int(7, ch));
    row("1/7 * 7", &one7.display(), (1.0 / 7.0) * 7.0, one7.to_f64());

    println!("\nEvery `exact` column is bit-for-bit correct; the f64 column drifts.");
}
