use aad::tape::Tape;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn large_computation_graph_benchmark(c: &mut Criterion) {
    c.bench_function("large_computation_graph", |b| {
        b.iter(|| {
            let tape = Tape::default();

            let x0 = tape.var(1.0);
            let x1 = tape.var(2.0);
            let x2 = tape.var(3.0);
            let x3 = tape.var(4.0);
            let x4 = tape.var(5.0);

            let mut result = x0;
            for i in 0..10000 {
                result = (((result + x1) * x2.sin()) + (x3 * x4.ln())) * (x2 + (i as f64).ln());
            }
            black_box(result);
            let grads = result.backward();
            black_box(grads.get(&[x0, x1, x2, x3, x4]));
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
                result = (((result + x1) * x2.sin()) + (x3 * x4.ln())) * (x2 + (i as f64).ln());
                black_box(result);
            }

            black_box(result);
        })
    });
}

fn large_computation_graph_benchmark_rustograd(c: &mut Criterion) {
    c.bench_function("large_computation_graph_rustograd", |b| {
        b.iter(|| {
            let tape = rustograd::Tape::default();

            let x0 = tape.term("x0", 1.0);
            let x1 = tape.term("x1", 2.0);
            let x2 = tape.term("x2", 3.0);
            let x3 = tape.term("x3", 4.0);
            let x4 = tape.term("x4", 5.0);

            let mut result = x0;
            for _ in 0..10000 {
                result = (((result + x1) * x2.apply("sin", f64::sin, f64::cos))
                    + (x3 * x4.apply("ln", f64::ln, f64::recip)))
                    * x2;
            }

            let _ = result.eval();

            result.backprop().unwrap();

            black_box(result.derive(&x0).unwrap());
            black_box(result.derive(&x1).unwrap());
            black_box(result.derive(&x2).unwrap());
            black_box(result.derive(&x3).unwrap());
            black_box(result.derive(&x4).unwrap());
        });
    });
}

criterion_group!(
    benches,
    large_computation_graph_benchmark,
    large_computation_graph_benchmark_rustograd,
    large_computation_graph_benchmark_f64
);
criterion_main!(benches);
