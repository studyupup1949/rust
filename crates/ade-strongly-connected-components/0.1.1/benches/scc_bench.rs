use ade_graph::utils::build::build_graph;
use ade_graph_generators::generate_random_graph_data;
use ade_strongly_connected_components::scc;
use ade_strongly_connected_components::scc_iterative;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn benchmark_strongly_connected_components(c: &mut Criterion) {
    let (nodes, edges) = generate_random_graph_data(20, 100, 123);
    let graph = build_graph(nodes, edges);

    c.bench_function("scc_20_nodes_100_edges", |b| {
        b.iter(|| {
            let components = scc(black_box(&graph));
            black_box(components.len())
        })
    });
}

fn benchmark_strongly_connected_components_iter(c: &mut Criterion) {
    let (nodes, edges) = generate_random_graph_data(20, 100, 123);
    let graph = build_graph(nodes, edges);

    c.bench_function("scc_20_nodes_100_edges_iter", |b| {
        b.iter(|| {
            let components = scc_iterative(black_box(&graph));
            black_box(components.len())
        })
    });
}

criterion_group!(
    benches,
    benchmark_strongly_connected_components,
    benchmark_strongly_connected_components_iter
);
criterion_main!(benches);
