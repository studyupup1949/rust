//! Exact structural-engineering fractions — no float drift in combination checks.
//!
//! Run with: `cargo run --example engineering`

use adele_ring::{Channels, RnsRational};

fn main() {
    let ch = Channels::standard(32);
    let f = |p: i64, q: i64| RnsRational::from_fraction(p, q, ch.clone());

    println!("== adele-ring :: exact engineering arithmetic ==\n");

    // Stacked plate thicknesses in inches: 3/8 + 1/4 + 5/16.
    let stack = f(3, 8).add(&f(1, 4)).add(&f(5, 16));
    println!("3/8 + 1/4 + 5/16 in   = {} in   (= {:.6} in)", stack, stack.to_f64());

    // Safety factor 1/1.5 = 2/3 exactly.
    let safety = f(1, 1).div(&f(3, 2));
    println!("1 / 1.5               = {}        (= {:.6})", safety, safety.to_f64());

    // Load ratio: 47 kips / 70 kips — stays exact through combination checks.
    let load_ratio = f(47, 70);
    let with_margin = load_ratio.add(&f(1, 10)); // add a 0.1 utilization margin
    println!(
        "47/70 + 1/10          = {}    (= {:.6})",
        with_margin,
        with_margin.to_f64()
    );

    // AISC-style web area: t_w * d  =  (5/16) * (24/1) in^2.
    let t_w = f(5, 16);
    let d = f(24, 1);
    let web_area = t_w.mul(&d);
    println!("(5/16) * 24           = {} in^2   (= {:.6} in^2)", web_area, web_area.to_f64());

    println!("\n-- base awareness --");
    for (p, q) in [(1, 6), (1, 8), (47, 70)] {
        let r = f(p, q);
        println!(
            "{p}/{q}: natural base = {}, terminates in base 10 = {}, period in base 10 = {}",
            r.natural_base(),
            r.exact_in_base(10),
            r.termination_period_in_base(10)
        );
    }

    // Demonstrate the classic float failure that adele-ring avoids.
    let exact = f(1, 10).add(&f(1, 5)); // 0.1 + 0.2
    let naive = 0.1_f64 + 0.2_f64;
    println!("\n0.1 + 0.2 exact       = {}   (f64 gives {:.17})", exact, naive);
    assert_eq!(exact, f(3, 10));
}
