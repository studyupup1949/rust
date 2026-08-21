//! Show Witt vector representation of a given adic

use std::iter::once;
use clap::Parser;
use adic::{
    error::AdicResult,
    normed::Valuation,
    traits::{AdicPrimitive, CanApproximate, HasDigits, PrimedFrom},
    EAdic, Variety, ZAdic,
};


#[derive(Parser)]
struct Args {
    #[arg(short)]
    p: u32,
    #[arg(short, allow_hyphen_values=true)]
    a: i32,
    #[arg(long)]
    precision: usize,
}

fn main() -> AdicResult<()> {
    let args = Args::parse();
    let p = args.p;
    let a = EAdic::primed_from(args.p, args.a);
    let precision = args.precision;

    println!("Witt vectors, 0 + roots of unity, for p={p} (i.e. solutions of x^p - x = 0):");

    let unity_roots = ZAdic::roots_of_unity(p, precision)?;
    let witt_vector_variety = Variety::new(once(ZAdic::zero(p)).chain(unity_roots.into_roots()).collect());
    println!("{witt_vector_variety}");

    println!("Convert adic number {a} = {} to Witt vector representation", args.a);

    let witt_vector = witt_vector_variety.into_roots().collect::<Vec<_>>();
    let mut representation = ZAdic::empty(p);
    let mut remainder = a.approximation(precision);
    for n in 0..precision {
        let next_digit = remainder.digit(0)?;
        representation.push_digit(next_digit)?;
        let new_remainder = remainder - witt_vector[next_digit as usize].clone();
        remainder = ZAdic::new_approx(p, precision - n - 1, new_remainder.digits().skip(1).collect());
    }

    println!("Witt representation ==> {representation} <==");

    println!("Show the work:");

    let display_a = a.into_approximation(precision);
    println!("{display_a}");
    println!("-----------");
    for (idx, witt_digit) in representation.digits().enumerate() {
        let mut witt_num = witt_vector[witt_digit as usize].clone();
        witt_num.set_certainty(Valuation::Finite(precision - idx));
        println!("{witt_num}");
    }

    Ok(())

}
