use aad::tape::Tape;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use RustQuant_autodiff::{Accumulate, Graph};

fn large_computation_graph_benchmark(c: &mut Criterion) {
    c.bench_function("large_computation_graph", |b| {
        b.iter(|| {
            let tape = Tape::default();

            let x0 = tape.create_variable(1.0);
            let x1 = tape.create_variable(2.0);
            let x2 = tape.create_variable(3.0);
            let x3 = tape.create_variable(4.0);
            let x4 = tape.create_variable(5.0);

            let mut result = x0;
            for i in 0..100000 {
                result += (((result + x1) * x2.sin()) + (x3 * x4.ln())) * (x2 + (i as f64).ln());
            }
            black_box(result);
            let grads = result.compute_gradients();
            black_box(grads);
        })
    });
}

fn large_computation_graph_benchmark_rust_quant(c: &mut Criterion) {
    c.bench_function("large_computation_graph_rust_quant", |b| {
        b.iter(|| {
            let tape = Graph::default();

            let x0 = tape.var(1.0);
            let x1 = tape.var(2.0);
            let x2 = tape.var(3.0);
            let x3 = tape.var(4.0);
            let x4 = tape.var(5.0);

            let mut result = x0;
            for i in 0..100000 {
                result += (((result + x1) * x2.sin()) + (x3 * x4.ln())) * (x2 + (i as f64).ln());
            }
            black_box(result);
            let grads = result.accumulate();
            black_box(grads);
        })
    });
}

fn large_computation_graph_benchmark_f64(c: &mut Criterion) {
    c.bench_function("large_computation_graph_f64", |b| {
        b.iter(|| {
            let x0 = 1.0_f64;
            let x1 = 2.0_f64;
            let x2 = 3.0_f64;
            let x3 = 4.0_f64;
            let x4 = 5.0_f64;

            let mut result = x0;
            for i in 0..10000 {
                result += (((result + x1) * x2.sin()) + (x3 * x4.ln())) * (x2 + (i as f64).ln());
                black_box(result);
            }

            black_box(result);
        })
    });
}

criterion_group!(
    benches,
    large_computation_graph_benchmark,
    large_computation_graph_benchmark_rust_quant,
    large_computation_graph_benchmark_f64
);
criterion_main!(benches);
