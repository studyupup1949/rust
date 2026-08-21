use adsbx_json::v2::Response;
use criterion::{criterion_group, criterion_main, Criterion};
use std::str::FromStr;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse", |b| {
        let input = include_str!("../tests/v2-specimen-01.json");
        b.iter(|| {
            Response::from_str(input).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
