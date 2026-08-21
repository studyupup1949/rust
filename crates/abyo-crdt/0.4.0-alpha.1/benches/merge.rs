//! Merge-cost benchmarks.
#![allow(missing_docs)]

use abyo_crdt::List;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_merge_disjoint(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_disjoint");
    for &size in &[10usize, 100, 1_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            // Pre-build two replicas with disjoint local edits.
            let mut a = List::<u32>::new(1);
            let mut b_replica = List::<u32>::new(2);
            for i in 0..n {
                a.insert(i, i as u32);
                b_replica.insert(i, (10_000 + i) as u32);
            }
            b.iter(|| {
                let mut a2 = a.clone();
                a2.merge(&b_replica);
                criterion::black_box(a2)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_merge_disjoint);
criterion_main!(benches);
