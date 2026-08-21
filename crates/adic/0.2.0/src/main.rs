//! # Adic
//!
//! Hensel lift algebraic varieties to the p-adic numbers

use clap::Parser;
use adic::{AdicInteger, RAdic};

#[derive(Parser)]
struct Args {
    #[arg(short)]
    p: u32,

    #[arg(short, allow_hyphen_values=true)]
    a: i32,

    #[arg(short)]
    n: u32,

    #[arg(long)]
    precision: u32,
}

fn main() {
    let args = Args::parse();
    let a = RAdic::from_integer(args.p, args.a);
    let out = a.nth_root(args.n, args.precision);

    match out {
        Ok(varieties) => println!("Varieties are {varieties:?}"),
        Err(err) => println!("{err:#?}")
    }
}
