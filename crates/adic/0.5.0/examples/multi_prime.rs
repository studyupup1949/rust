//! Create and manipulate multi-prime adics, e.g. 10-adics

use clap::Parser;
use itertools::Itertools;
use adic::{
    divisible::Composite,
    error::AdicResult,
    num_adic::MAdic,
    traits::HasDigits,
    ZAdic,
};


#[derive(Parser)]
struct Args {

    #[arg(short)]
    mp: u32,

    #[arg(short, allow_hyphen_values=true)]
    a: i32,

    #[arg(long)]
    precision: usize,

}


fn main() -> AdicResult<()> {

    let args = Args::parse();
    let ma = MAdic::approx_from_i32(args.mp, args.a, args.precision)?;
    println!("{ma}");

    let digit_vec = ma.digits().collect::<Vec<_>>();
    println!("Digits:");
    println!("{}", digit_vec.into_iter().rev().map(|d|
        if d < 10 {
            d.to_string()
        } else {
            format!("[{d}]")
        }
    ).collect::<String>());

    let c = Composite::from(args.mp);

    println!("Prime idempotents:");
    let primes = c.primes();
    let prime_idemps = MAdic::<ZAdic>::prime_idempotents(args.mp, args.precision)?;
    for (prime, idemp) in primes.zip(prime_idemps) {
        println!("Prime {prime}, idemp {}", idemp.to_str_radix(args.mp));
    }

    println!("All idempotents:");
    let primes = c.primes();
    let all_idemps = MAdic::<ZAdic>::all_idempotents(args.mp, args.precision)?;
    for (primes, idemp) in primes.powerset().zip(all_idemps) {
        println!("Primes {}, idemp {}", primes.into_iter().join(" x "), idemp.to_str_radix(args.mp));
    }

    Ok(())

}
