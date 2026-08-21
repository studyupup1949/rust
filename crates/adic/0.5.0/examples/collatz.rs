//! Collatz investigations

use adic::{
    normed::Normed,
    traits::{AdicInteger, AdicPrimitive, PrimedFrom},
    EAdic, QAdic, UAdic,
};
use itertools::Itertools;

fn main() {

    println!("INTEGERS");
    println!("====\n");

    for n in 0i32..20 {

        let adic_n = EAdic::primed_from(2, n);

        println!("n: {n}, adic_n: {adic_n}");
        let _endpoint = collatz_iteration(adic_n);
        println!("");

    }

    println!("RATIONALS");
    println!("====\n");

    let mut visited = vec![];
    for (a, b) in (-7..7).cartesian_product((1..7)) {

        // Note: we want to use `from_rational` here, but we need to change `from_rational` to return a QAdic instead
        let adic_full_ab = QAdic::<EAdic>::primed_from(2, a) / QAdic::<EAdic>::primed_from(2, b);
        let adic_unit_ab = adic_full_ab.unit().unwrap_or(EAdic::zero(2));

        if visited.contains(&adic_unit_ab) {
            // println!("SKIP, adic already calculated")
        } else {
            println!("a/b: {a}/{b}, adic_ab: {adic_full_ab}");
            visited.push(adic_unit_ab.clone());
            let _endpoint = collatz_iteration(adic_unit_ab);
            println!("");
        }

    }

}

fn collatz_iteration<A>(input_adic: A) -> A
where A: AdicInteger + std::cmp::PartialEq {

    let mut cycle_length = 1;
    let mut x = divide_out_twos(input_adic);
    let mut visited_list = vec![x.clone()];
    while !x.is_local_zero() {
        x = A::from(UAdic::new(2, vec![1, 1])) * x + A::one(2);
        x = divide_out_twos(x);
        if visited_list.contains(&x) {
            let x_position = visited_list.iter().position(|y| *y == x);
            cycle_length = visited_list.len() - x_position.unwrap();
            break;
        } else {
            visited_list.push(x.clone());
        }
    }

    if x.is_local_zero() {
        println!("{}", visited_list.iter().join(" --> "));
        println!("Zero terminating");
    } else {
        let repeating_list = visited_list.split_off(visited_list.len() - cycle_length);
        let dash_str = " --> ";
        println!(
            "{}[[[ {} --> ]]]",
            if visited_list.is_empty() { String::new() } else { visited_list.iter().join(&dash_str) + &dash_str },
            repeating_list.iter().join(&dash_str) + &dash_str
        );
        println!("cycle length: {cycle_length}");
    }

    x

}


fn divide_out_twos<A>(x: A) -> A
where A: AdicInteger {
    x.into_unit().unwrap_or(A::zero(2))
}
