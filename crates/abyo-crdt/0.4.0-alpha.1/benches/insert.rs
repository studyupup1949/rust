//! Insertion-rate benchmarks.
#![allow(missing_docs)]

use abyo_crdt::List;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("append");
    for &size in &[10usize, 100, 1_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            b.iter(|| {
                let mut list = List::<u32>::new(1);
                for i in 0..n {
                    list.insert(i, i as u32);
                }
                criterion::black_box(list)
            });
        });
    }
    group.finish();
}

fn bench_prepend(c: &mut Criterion) {
    let mut group = c.benchmark_group("prepend");
    for &size in &[10usize, 100, 1_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            b.iter(|| {
                let mut list = List::<u32>::new(1);
                for i in 0..n {
                    list.insert(0, i as u32);
                }
                criterion::black_box(list)
            });
        });
    }
    group.finish();
}

fn bench_iter_after_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_after_build");
    for &size in &[100usize, 1_000, 10_000] {
        // Pre-build the doc once.
        let mut list = List::<u32>::new(1);
        for i in 0..size {
            list.insert(i, i as u32);
        }
        group.bench_with_input(BenchmarkId::from_parameter(size), &list, |b, list| {
            // Each iteration: full traversal. After the first, the cache is hot.
            b.iter(|| {
                let n: u64 = list.iter().map(|&v| u64::from(v)).sum();
                criterion::black_box(n)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_append, bench_prepend, bench_iter_after_build);
criterion_main!(benches);
