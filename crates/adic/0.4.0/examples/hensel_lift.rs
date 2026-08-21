//! # Adic
//!
//! Hensel lift algebraic varieties to the p-adic numbers

use std::iter::once;
use std::iter::repeat_n;
use clap::Parser;
use adic::{AdicInteger, AdicPolynomial, IAdic, SignedAdicNumber};

#[derive(Parser)]
struct Args {
    #[arg(short)]
    p: u32,

    #[arg(short, allow_hyphen_values=true)]
    a: i32,

    #[arg(short)]
    n: u32,

    #[arg(long)]
    precision: usize,
}

fn main() {
    let args = Args::parse();

    let a = IAdic::from_i32(args.p, args.a);
    let out_nth_root = a.nth_root(args.n, args.precision);

    match &out_nth_root {
        Ok(varieties) => {
            println!("n-th roots are {varieties:?}");
        },
        Err(err) => {
            println!("{err:#?}");
        },
    };

    let coeffs = Vec::from_iter(
        once(-args.a)
        .chain(repeat_n(0, args.n as usize - 1))
        .chain(once(1))
        .map(|b| IAdic::from_i32(args.p, b))
    );
    let out_variety = AdicPolynomial::new(args.p, coeffs).variety(args.precision);

    assert_eq!(out_nth_root, out_variety);

    match out_variety {
        Ok(varieties) => println!("Varieties are {varieties:?}"),
        Err(err) => println!("{err:#?}")
    }
}
