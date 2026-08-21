//! Look how roots of root varieties behave


use clap::Parser;
use adic::{AdicError, AdicInteger, IAdic, SignedAdicInteger, ZAdic, ZAdicValuation};


#[derive(Parser)]
struct Args {
    #[arg(short)]
    p: u32,
    #[arg(short, allow_hyphen_values=true)]
    a: i32,
}


fn printrrt<AI>(adic_int: &AI, n: u32, precision: usize) -> Result<(), AdicError>
where AI: AdicInteger {
    let vars = adic_int.nth_root(n, precision)?;
    if vars.is_empty() {
        println!("rt{n}({adic_int}): EMPTY");
    } else {
        println!("rt{n}({adic_int}): {}", adic_int.nth_root(n, 10)?);
    }
    Ok(())
}

fn printzrt(adic_int: &ZAdic, n: u32, precision: usize) -> Result<(), AdicError> {
    let vars = adic_int.nth_root(n, precision)?;
    let mut trunced_int = adic_int.clone();
    trunced_int.set_certainty(std::cmp::min(adic_int.certainty(), ZAdicValuation::Finite(10)));
    if vars.is_empty() {
        println!("rt{n}({trunced_int}): EMPTY");
    } else {
        println!("rt{n}({trunced_int}): {}", adic_int.nth_root(n, 10)?);
    }
    Ok(())
}


fn main() -> Result<(), AdicError> {
    let args = Args::parse();
    let a = &IAdic::from_i32(args.p, args.a);

    println!("Varieties of roots 1 to 15");

    printrrt(a, 1, 10)?;
    printrrt(a, 2, 10)?;
    printrrt(a, 3, 10)?;
    printrrt(a, 4, 10)?;
    printrrt(a, 5, 10)?;
    printrrt(a, 6, 10)?;
    printrrt(a, 7, 10)?;
    printrrt(a, 8, 10)?;
    printrrt(a, 9, 10)?;
    printrrt(a, 10, 10)?;
    printrrt(a, 11, 10)?;
    printrrt(a, 12, 10)?;
    printrrt(a, 13, 10)?;
    printrrt(a, 14, 10)?;
    printrrt(a, 15, 10)?;

    println!();
    println!("Varieties of combo roots 1 to 15");

    for var in a.nth_root(2, 70)?.roots() {
        println!("for rt2 var {var}");
        printzrt(var, 2, 10)?;
        printzrt(var, 3, 10)?;
        printzrt(var, 4, 10)?;
        printzrt(var, 5, 10)?;
        printzrt(var, 6, 10)?;
        printzrt(var, 7, 10)?;
    }

    for var in a.nth_root(3, 40)?.roots() {
        println!("for rt3 var {var}");
        printzrt(var, 2, 10)?;
        printzrt(var, 3, 10)?;
        printzrt(var, 4, 10)?;
    }

    for var in a.nth_root(4, 30)?.roots() {
        println!("for rt4 var {var}");
        printzrt(var, 2, 10)?;
        printzrt(var, 3, 10)?;
    }

    for var in a.nth_root(5, 30)?.roots() {
        println!("for rt5 var {var}");
        printzrt(var, 2, 10)?;
        printzrt(var, 3, 10)?;
    }

    for var in a.nth_root(6, 30)?.roots() {
        println!("for rt6 var {var}");
        printzrt(var, 2, 10)?;
    }

    for var in a.nth_root(7, 30)?.roots() {
        println!("for rt7 var {var}");
        printzrt(var, 2, 10)?;
    }

    Ok(())

}
