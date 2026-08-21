//! # Adic
//!
//! Hensel lift algebraic varieties to the p-adic numbers

use clap::Parser;
use adic::variety_to_digits;

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
    let out = variety_to_digits(args.p, args.a, args.n, args.precision);

    match out {
        Ok(varieties) => println!("Varieties are {:?}", varieties),
        Err(err) => println!("{:#?}", err)
    }
}
