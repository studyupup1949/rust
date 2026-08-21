use std::env;
use std::str::FromStr;
use std::string::ToString;
use std::time::Instant;
use num_bigint::BigUint;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion};
use criterion::measurement::WallTime;
use num_traits::ToPrimitive;
use accumulator_plus::{batch_add_big_uint, batch_auth_by_normal_loop, batch_auth_by_shamir_trick, batch_delete_by_normal_loop, batch_delete_by_shamir_trick, gen_diff_prime_vec, MODULUS};
use accumulator_plus::proofs::{poe_big_uint, verify_poe_big_uint};
use accumulator_plus::subroutines::mod_exp_big_uint;
use accumulator_plus::witnesses::create_all_mem_wit;


fn config_parse(config: &mut Config) {
    let mut args = env::args();
    let _ = args.next();

    match args.next() {
        None => panic!("需要第一个数值型参数： g_k <表示分域的数量> ！"),
        Some(gk) => config.g_k = gk.parse().unwrap()
    }
    match args.next() {
        None => panic!("需要第二个数值型参数： g_bits <表示一个分域标识需要占用bit数> ！"),
        Some(gk) => config.g_bits = gk.parse().unwrap()
    }
    match args.next() {
        None => panic!("需要第三个数值型参数： e_k <表示一个分域内假名的数量> ！"),
        Some(gk) => config.e_k = gk.parse().unwrap()
    }
    match args.next() {
        None => panic!("需要第四个数值型参数： e_bits <表示一个假名标识需要占用bit数> ！"),
        Some(gk) => config.e_bits = gk.parse().unwrap()
    }
    match args.next() {
        None => panic!("需要第五个数值型参数： delete_n <表示一次批量处理认证/删除假名数量> ！"),
        Some(gk) => config.batch_interval_percent = gk.parse().unwrap()
    }
    match args.next() {
        None => {
            // println!("可选第六个字符型参数： delete_mod <表示一次批量处理认证/删除的模式，可选模式[loop/shamir]>, 默认shamir模式 ！");
        }
        Some(gk) => config.batch_mod = gk.parse().unwrap()
    }
}

fn init(tag: String, g_k: usize, g_bits: usize, e_k: usize, e_bits: usize, s: &mut Vec<BigUint>, vpks: &mut Vec<BigUint>) {
    println!("{}", tag.clone() + "/Gi gen <loop>");
    let now = Instant::now();
    gen_diff_prime_vec(g_k, g_bits, s);
    println!("\t\t\t\t time: {:?} ms", now.elapsed().as_millis());  // 1

    println!("{}", tag.clone() + "/Ei list gen <loop>");
    let now = Instant::now();
    gen_diff_prime_vec(e_k, e_bits, vpks);
    println!("\t\t\t\t time: {:?} ms", now.elapsed().as_millis());  //2

}


fn run_count(c: &mut Criterion, tag: String, config: Config) {
    let mut group = c.benchmark_group(tag.clone());
    group.significance_level(0.1).sample_size(10);

    // 初始化
    //-------------------------------
    let mut s: Vec<BigUint> = Vec::new();
    let mut vpks: Vec<BigUint> = Vec::new();
    init(tag.clone(), config.g_k, config.g_bits, config.e_k, config.e_bits, &mut s, &mut vpks);
    //-------------------------------

    let old_state = s.get(0).unwrap();
    println!("gi list len: {}", s.len());
    let new_elems = vpks.clone();
    println!("ei list len: {}", new_elems.len());

    // 节点值的离线生成
    //-------------------------------
    group.bench_function("Si gen <batch>", |b| b.iter(|| {
        let (_, _, _) = batch_add_big_uint(&old_state, &new_elems);
    }));
    let (new_state, x_agg, _) = batch_add_big_uint(&old_state, &new_elems);
    println!("si size : {:?}", new_state);
    println!("x_agg size : {:?}", x_agg.to_bytes_le().len());

    // 响应每个假名，见证值
    //-------------------------------
    group.bench_function("Wi list gen <RootFactor>", |b| b.iter(|| {
        let _ = create_all_mem_wit(&old_state, &new_elems); // 3
    }));
    let vms = create_all_mem_wit(&old_state, &new_elems); // 3
    println!("wi list len: {}", &vms.len());

    let percent = config.batch_interval_percent;
    let n = 100 / percent;
    for i in 0..n {
        let batch_n = (i + 1) * percent;
        // -------------------------------------------------
        // 批量认证
        //-------------------------------
        let batch_n_auth_data = new_elems[0..batch_n].iter().cloned().collect();
        let batch_n_auth_stat = new_state.clone();
        group.bench_function(format!("BATCH_{}/Authentication <{}>", batch_n, config.batch_mod),
                             |b| b.iter(|| {  // 6
                                 if config.batch_mod == config.mod_default_key {
                                     batch_auth_by_normal_loop(&batch_n_auth_data, &vms, &batch_n_auth_stat);
                                 } else {
                                     batch_auth_by_shamir_trick(&batch_n_auth_data, &vms, &batch_n_auth_stat);
                                 }
                             }));
        // -------------------------------------------------
        // 批量删除
        //-------------------------------
        let pairs: Vec<(BigUint, BigUint)> = new_elems.iter().cloned().zip(vms.iter().cloned()).collect();
        let dels: Vec<(BigUint, BigUint)> = pairs[0..batch_n].iter().cloned().collect();
        let elems = vpks.clone();

        group.bench_function(format!("BATCH_{}/Delete <{}>", batch_n, config.batch_mod),
                             |b| b.iter(|| {  // 6
                                 if config.batch_mod == config.mod_default_key {
                                     let rest = batch_delete_by_normal_loop(old_state.clone(), new_state.clone(), &dels, &elems);
                                     black_box(rest);
                                 } else {
                                     let rest = batch_delete_by_shamir_trick(new_state.clone(), &dels);
                                     black_box(rest);
                                 }
                             }));
        let rest;
        if config.batch_mod == config.mod_default_key {
            rest = batch_delete_by_normal_loop(old_state.clone(), new_state.clone(), &dels, &elems);
        } else {
            rest = batch_delete_by_shamir_trick(new_state.clone(), &dels);
        }
        // -------------------------------------------------
        // CHECK FUNC
        //-------------------------------
        group.bench_function(format!("MOD_EXP ACTION BATCH {}", batch_n), |b| b.iter(|| {  // 6
            let state = mod_exp_big_uint(&rest.0, &rest.1, &BigUint::from_str(MODULUS).unwrap());
            black_box(state);
        }));
        assert_eq!(mod_exp_big_uint(&rest.0, &rest.1, &BigUint::from_str(MODULUS).unwrap()), new_state.clone());
        // -------------------------------------------------
        // TODO-------------------------------------------------
        // let elems = &x_agg % &BigUint::from_str(MODULUS).unwrap();
        let elems = x_agg.clone();
        // -------------------------------------------------
        group.bench_function(format!("POE ACTION BATCH {}", batch_n), |b| b.iter(|| {  // 7
            let proof = poe_big_uint(&rest.0, &elems, &new_state);
            black_box(proof);
        }));
        let proof = poe_big_uint(&rest.0, &elems, &new_state);
        if verify_poe_big_uint(&rest.0, &elems, &new_state, &proof) == true {
            println!("poe ok!");
        }
    }


    group.finish();
}

pub const NORMAL_MOD:&str = "normal";

#[derive(Clone)]
struct Config {
    g_k: usize,
    g_bits: usize,
    e_k: usize,
    e_bits: usize,
    batch_interval_percent: usize,
    batch_mod: String,
    mod_default_key: String,
}
impl Config {
    fn new()->Config{
        Config{
            mod_default_key: NORMAL_MOD.to_string(),
            g_k: 1,
            g_bits: 256,
            e_k: 100,
            e_bits: 256,
            batch_interval_percent: 50,
            batch_mod: NORMAL_MOD.to_string(),
        }
    }
}

fn bench_accumulator(c: &mut Criterion) {
    let config = Config::new();
    // TODO mod switch
    // config.batch_mod = "shamir".to_string();


    println!("\r\n-------------------------------------start test: -------------------------------------\r\n");
    // config_parse(&mut config);
    run_count(c, format!("bench_accumulator_ini<si:{}/{}><ei:{}/{}>", config.g_bits, config.g_k, config.e_bits, config.e_k), config.clone());
}

criterion_group!(benches, bench_accumulator);
criterion_main!(benches);





