//! # Adic
//!
//! Hensel lift algebraic varieties to the p-adic numbers

use std::iter::once;
use std::iter::repeat_n;
use clap::Parser;
use adic::{
    traits::{AdicInteger, PrimedFrom},
    EAdic, Polynomial,
};

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

    let a = EAdic::primed_from(args.p, args.a);
    let out_nth_root = a.nth_root(args.n, args.precision);

    match &out_nth_root {
        Ok(variety) => {
            println!("n-th root variety is {variety}");
        },
        Err(err) => {
            println!("{err:#?}");
        },
    };

    // x^n - a
    let coeffs = Vec::from_iter(
        once(-args.a)
        .chain(repeat_n(0, args.n as usize - 1))
        .chain(once(1))
        .map(|b| EAdic::primed_from(args.p, b))
    );
    let out_variety = Polynomial::<EAdic>::new(coeffs).variety(isize::try_from(args.precision).unwrap());

    match out_variety {
        Ok(variety) => {
            assert!(out_nth_root.is_ok());
            assert_eq!(out_nth_root, variety.clone().try_into_integer());
            println!("Polynomial variety is {variety}")
        },
        Err(err) => println!("{err:#?}")
    }
}
