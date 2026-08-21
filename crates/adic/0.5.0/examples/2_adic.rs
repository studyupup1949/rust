//! Investigate 2-adic peculiarities

use num::{traits::Pow, Rational32};
use adic::{
    function::{factory, special},
    mapping::IndexedMapping,
    traits::{AdicInteger, AdicPrimitive, CanApproximate, CanTruncate, PrimedFrom},
    EAdic, QAdic, Variety, ZAdic,
};

fn main() {

    println!("Basic 2-adic");
    println!("");

    let neg_one = ZAdic::primed_from(2, -1);
    println!("Negative one: {neg_one}");

    // The precisions below were chosen to avoid costly division operations; this will be improved over time

    let log_neg_one = special::iwasawa_log(QAdic::from_integer(neg_one), 8).unwrap();
    println!("Log(-1): {log_neg_one}");

    let manual_log_neg_one = (1..20).map(
        |n| -(EAdic::new(2, vec![0, 1]).pow(n) / EAdic::primed_from(2, n)).approximation(8)
    ).fold(ZAdic::zero(2), |acc, x| acc + x);
    println!("Log(-1), manual: {manual_log_neg_one}");

    let log_7 = special::iwasawa_log(QAdic::from_integer(EAdic::new(2, vec![1, 1, 1])), 8).unwrap();
    println!("Log(7 ~= -1): {log_7}");

    let log_15 = special::iwasawa_log(QAdic::from_integer(EAdic::new(2, vec![1, 1, 1, 1])), 8).unwrap();
    println!("Log(15 ~= -1): {log_15}");

    let log_63 = special::iwasawa_log(QAdic::from_integer(EAdic::new(2, vec![1, 1, 1, 1, 1, 1])), 8).unwrap();
    println!("Log(63 ~= -1): {log_63}");

    let one_fifth = QAdic::<EAdic>::primed_from(2, Rational32::new(1, 5));
    println!("1/5: {one_fifth}");
    let neg_one_fifth = QAdic::<EAdic>::primed_from(2, Rational32::new(-1, 5));
    println!("-1/5: {neg_one_fifth}");

    let log_1_5 = special::iwasawa_log(one_fifth, 8).unwrap();
    println!("Log(1/5): {log_1_5}");

    let exp_log_1_5 = factory::exp().eval_finite(log_1_5, 8).unwrap();
    println!("exp(Log(1/5)): {exp_log_1_5}");

    println!("");
    println!("Artin-Hasse unit function");
    println!("");

    for n in 1..=20 {
        artin_hasse_unit(QAdic::primed_from(2, n), 8);
    }

}


fn artin_hasse_unit(x: QAdic<ZAdic>, precision: usize) -> Variety<ZAdic> {

    println!("");
    println!("artin_hasse_unit({x})");
    println!("");

    let log_x = special::iwasawa_log(x, precision + 1).unwrap();
    let log_sq_x = log_x.clone().pow(2);
    println!("log(x) = {log_x}");
    println!("log(x)^2 = {log_sq_x}");

    let ah_2_exp_log_x = artin_hasse_truncated_2(log_x.clone(), precision);
    println!("Truncated Artin-Hasse (to 2) E_2(log(x)) = {ah_2_exp_log_x}");

    let exp_log_sq_x = factory::exp().eval_finite(log_sq_x, precision + 1).unwrap();
    let sqrt_exp_log_sq_x = exp_log_sq_x.split(0).1.nth_root(2, precision).unwrap();
    println!("sqrt(exp(log(x)^2)) = {sqrt_exp_log_sq_x}");

    let roots = sqrt_exp_log_sq_x.into_roots().collect::<Vec<_>>();
    let unit_0 = (ah_2_exp_log_x.clone() / roots[0].clone());
    let unit_1 = (ah_2_exp_log_x.clone() / roots[1].clone());
    println!("unit_0(x) = {unit_0}");
    println!("unit_1(x) = {unit_1}");

    Variety::new(vec![unit_0, unit_1])

}


fn artin_hasse_truncated_2(x: QAdic<ZAdic>, precision: usize) -> ZAdic {
    let x_sq_over_2 = (x.clone() * x.clone()) / QAdic::primed_from(2, 2);
    println!("{}, {}", x, x_sq_over_2);
    factory::exp().eval_finite(x + x_sq_over_2, precision).unwrap().split(0).1
}
