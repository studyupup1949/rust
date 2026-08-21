use criterion::{Criterion, black_box, criterion_group, criterion_main};

use abaco::{Evaluator, UnitRegistry};

fn bench_eval_simple(c: &mut Criterion) {
    let eval = Evaluator::new();
    let mut group = c.benchmark_group("eval_simple");

    group.bench_function("addition", |b| b.iter(|| eval.eval(black_box("2 + 3"))));
    group.bench_function("mixed_ops", |b| {
        b.iter(|| eval.eval(black_box("2 + 3 * 4")))
    });
    group.bench_function("parentheses", |b| {
        b.iter(|| eval.eval(black_box("(2 + 3) * 4")))
    });
    group.bench_function("division", |b| b.iter(|| eval.eval(black_box("10 / 3"))));

    group.finish();
}

fn bench_eval_functions(c: &mut Criterion) {
    let eval = Evaluator::new();
    let mut group = c.benchmark_group("eval_functions");

    group.bench_function("sqrt", |b| b.iter(|| eval.eval(black_box("sqrt(16)"))));
    group.bench_function("sin", |b| b.iter(|| eval.eval(black_box("sin(3.14159)"))));
    group.bench_function("log2", |b| b.iter(|| eval.eval(black_box("log2(1024)"))));
    group.bench_function("pow", |b| b.iter(|| eval.eval(black_box("pow(2, 10)"))));
    group.bench_function("min", |b| b.iter(|| eval.eval(black_box("min(3, 5)"))));

    group.finish();
}

fn bench_eval_complex(c: &mut Criterion) {
    let eval = Evaluator::new();
    let mut group = c.benchmark_group("eval_complex");

    group.bench_function("mixed_ops_funcs", |b| {
        b.iter(|| eval.eval(black_box("2^3 + sqrt(9) * 2")))
    });
    group.bench_function("nested_parens", |b| {
        b.iter(|| eval.eval(black_box("((1 + 2) * (3 + 4)) / (5 - 1)")))
    });
    group.bench_function("trig_chain", |b| {
        b.iter(|| eval.eval(black_box("sin(pi / 4) * cos(pi / 4)")))
    });
    group.bench_function("long_addition", |b| {
        b.iter(|| eval.eval(black_box("1+2+3+4+5+6+7+8+9+10")))
    });

    group.finish();
}

fn bench_eval_scientific(c: &mut Criterion) {
    let eval = Evaluator::new();
    let mut group = c.benchmark_group("eval_scientific");

    group.bench_function("sci_add", |b| {
        b.iter(|| eval.eval(black_box("1.5e10 + 2.3e9")))
    });
    group.bench_function("sci_mul", |b| b.iter(|| eval.eval(black_box("1e-3 * 1e3"))));

    group.finish();
}

fn bench_tokenizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");

    group.bench_function("simple", |b| {
        b.iter(|| abaco::tokenize(black_box("2 + 3 * 4")))
    });
    group.bench_function("complex", |b| {
        b.iter(|| abaco::tokenize(black_box("sin(pi / 4) * cos(pi / 4) + sqrt(abs(-16)) ^ 2")))
    });

    group.finish();
}

fn bench_unit_conversion(c: &mut Criterion) {
    let reg = UnitRegistry::new();
    let mut group = c.benchmark_group("unit_conversion");

    group.bench_function("km_to_miles", |b| {
        b.iter(|| reg.convert(black_box(100.0), "km", "mi"))
    });
    group.bench_function("celsius_to_fahrenheit", |b| {
        b.iter(|| reg.convert(black_box(100.0), "celsius", "fahrenheit"))
    });
    group.bench_function("bytes_to_gb", |b| {
        b.iter(|| reg.convert(black_box(1_000_000_000.0), "byte", "GB"))
    });
    group.bench_function("bytes_to_gib", |b| {
        b.iter(|| reg.convert(black_box(1_073_741_824.0), "byte", "GiB"))
    });
    group.bench_function("gb_to_gib_cross", |b| {
        b.iter(|| reg.convert(black_box(1.0), "GB", "GiB"))
    });
    group.bench_function("same_unit_identity", |b| {
        b.iter(|| reg.convert(black_box(42.0), "km", "km"))
    });

    group.finish();
}

fn bench_unit_lookup(c: &mut Criterion) {
    let reg = UnitRegistry::new();
    let mut group = c.benchmark_group("unit_lookup");

    group.bench_function("exact_symbol", |b| {
        b.iter(|| reg.find_unit(black_box("km")))
    });
    group.bench_function("by_name", |b| {
        b.iter(|| reg.find_unit(black_box("kilometer")))
    });
    group.bench_function("case_insensitive", |b| {
        b.iter(|| reg.find_unit(black_box("Kilometer")))
    });
    group.bench_function("plural", |b| b.iter(|| reg.find_unit(black_box("meters"))));
    group.bench_function("miss", |b| {
        b.iter(|| reg.find_unit(black_box("nonexistent")))
    });

    group.finish();
}

fn bench_registry_creation(c: &mut Criterion) {
    c.bench_function("registry_creation", |b| b.iter(UnitRegistry::new));
}

criterion_group!(
    benches,
    bench_eval_simple,
    bench_eval_functions,
    bench_eval_complex,
    bench_eval_scientific,
    bench_tokenizer,
    bench_unit_conversion,
    bench_unit_lookup,
    bench_registry_creation,
);
criterion_main!(benches);
