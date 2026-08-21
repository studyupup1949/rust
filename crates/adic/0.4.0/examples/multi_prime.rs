//! Create and manipulate multi-prime adics, e.g. 10-adics


use clap::Parser;
use itertools::Itertools;
use num::{traits::Pow, BigUint};
use num_prime::nt_funcs::factorize;
use adic::{special_function::carmichael, AdicComposite, AdicResult, HasDigits};


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
    let ma = AdicComposite::approx_from_i32(args.mp, args.a, args.precision)?;
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

    let idemps = idempotents(args.mp, args.precision);
    println!("idemps: {idemps:?}");

    for idemp in idemps {
        let idemp_in_base = idemp.to_str_radix(args.mp);
        println!("idemp {idemp_in_base}");
    }

    Ok(())

}



// These ought to be lazy static if possible!
fn idempotents(base: u32, precision: usize) -> Vec<BigUint> {

    // There are idempotents in multi-adics, zero-like and one-like numbers.
    // These satisfy the property T^2 = T.
    // For pure p-adics or p^n-adics, only 0 and 1 satisfy these.
    // For mixed n-adics, there are more.

    // For multi-adics, you can convert down from n-adic to p-adic, a surjective (onto) function.
    // E.g. the 10-adic 523._10 converts to 3 + 2*10 + 5*100 = 3 + 4*5 + 4*5^3 = 4043._5.
    // But again, this is SURJECTIVE, and e.g. multiple numbers convert to 0.
    // In particular, the number T_5 = lim_n->inf 5^(2^n) is a valid 10-adic but converts to the 5-adic 0.
    // This particular number also converts to the 2-adic 1, since 5^(2^n) -> 1 mod 2^m (Fermat/Euler?)
    // And subtracting T_5 from 1 gives the other idempotent: T_2 = 1 - T_5.

    // T_5 = ...59918212890625_10 -> 0._5 & 1._2
    // T_2 = ...40081787109376_10 -> 1._5 & 0._2

    // Note that these have the properties: T_2 * T_5 = 0; T_2 + T_5 = 1

    // In general, we want numbers that behave like 0 for one p-adic and 1 for the others.
    // This way, we get the idempotents:
    // - 0, 1
    // - T_p, for distinct prime p in n
    // - \prod_p T_p excluding the full product, since that gives 0

    // Or perhaps another way to think of it, the product of any number of T_p from the primes:
    // - The empty product gives 1
    // - The single product gives each "prime idempotent" separately, T_p
    // - The composite products multiply the prime idempotents once each, T_p1 * T_p2 * T_p3
    // - The full product gives 0

    // So we just need to calculate the prime idempotents and then multiply them together judiciously.
    // The prime idempotents should just be lim_n->inf (p1)^((p2*p3*...)^n)

    // Or more properly, if the adic has base b = p0^k0 p1^k1 p2^k2 ...
    // T_p0 = lim_N->inf (p0^k0)^carmichael( (p1^k1 p2^k2 ...)^N )

    let prime_factors = factorize(base);
    let prec32 = u32::try_from(precision).unwrap();
    let base_modulus = BigUint::from(base).pow(prec32);

    // Find the single-prime idempotents: T_p
    let idempotent_generators = prime_factors.iter().map(|(p, p_pow)| {
        let p_pow32 = u32::try_from(*p_pow).expect("usize -> u32 conversion");
        let others = base / p.pow(p_pow32);
        prime_idempotent(*p, p_pow32, others, precision)
    });

    let idempotent_nums = idempotent_generators
        .powerset()
        .map(|ps| ps.into_iter().product::<BigUint>() % base_modulus.clone())
        .collect::<Vec<_>>();

    idempotent_nums

}


fn prime_idempotent(p: u32, p_pow: u32, others: u32, precision: usize) -> BigUint {

    let prime_power = p.pow(u32::try_from(p_pow).expect("usize -> u32 conversion"));
    let prec32 = u32::try_from(precision).unwrap();
    let base_modulus = BigUint::from(p.pow(p_pow) * others).pow(prec32);
    let others_modulus = BigUint::from(others).pow(prec32);
    let carm = carmichael(others_modulus);
    let mod_pow = BigUint::from(prime_power).modpow(&carm, &base_modulus.clone());
    mod_pow

}
