use ades::{logic::{ades::ades, aes::aes, des::des}, cli::{TimeUnit, get_args, Algorithm}};
use aoko::{no_std::functions::ext::AnyExt1, standard::functions::fun::{measure_time_with_value, time_conversion_with_unit}};
use std::time::Duration;

fn algo() -> (impl FnOnce(Duration) -> u128, TimeUnit) {
    // 解析命令行参数
    let (r#in, out, aes_key, des_key, unit, subcmd) = get_args().let_owned(|s| (s.input, s.output, s.aes_key, s.des_key, s.time, s.subcmd));
    // 判断算法和加解密(子命令)
    use Algorithm::*;
    match subcmd {
        M(it) => ades(r#in, out, aes_key, des_key, it.encrypt),
        AES(it) => aes(r#in, out, aes_key, it.encrypt),
        DES(it) => des(r#in, out, des_key, it.encrypt),
    }
    // 返回转换函数和计时单位
    time_conversion_with_unit(unit)
}

fn main() {
    measure_time_with_value(algo)
        .let_owned(|((f, u), e)| println!("Execution time: {} {:?}.", f(e), u));
}