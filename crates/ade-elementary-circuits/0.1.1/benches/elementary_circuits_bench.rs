use ade_elementary_circuits::*;
use ade_graph::utils::build::build_graph;
use ade_graph_generators::complete_graph_data;
use ade_graph_generators::generate_random_graph_data;
use criterion::{criterion_group, criterion_main, Criterion};
use graph_cycles::Cycles;
use petgraph::graph::Graph;
use std::collections::HashSet;
use std::hint::black_box;

fn benchmark_elementary_circuits_single(c: &mut Criterion) {
    // Pre-generate the complete graph outside the benchmark
    let n: usize = 4;
    let (nodes, edges) = complete_graph_data(n);
    let graph = build_graph(nodes, edges);

    c.bench_function("elementary_circuits_n4", |b| {
        b.iter(|| {
            let circuits = elementary_circuits(black_box(&graph));
            black_box(circuits.len())
        })
    });
}

fn benchmark_elementary_random_graph(c: &mut Criterion) {
    let (nodes, edges) = generate_random_graph_data(11, 44, 3);
    let graph = build_graph(nodes, edges);

    c.bench_function("elementary_circuits_random_graph", |b| {
        b.iter(|| {
            let circuits = elementary_circuits(black_box(&graph));
            black_box(circuits.len())
        })
    });
}

criterion_group!(
    benches,
    benchmark_elementary_circuits_single,
    benchmark_elementary_random_graph,
);
criterion_main!(benches);
