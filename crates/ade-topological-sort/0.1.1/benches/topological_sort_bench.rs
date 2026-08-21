use ade_graph::implementations::{Edge, Node};
use ade_graph::utils::build::build_graph;
use ade_graph_generators::generate_random_graph_data;
use ade_topological_sort::topological_sort;
use ade_traits::NodeTrait;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

// Run scc_bench benchmark
// cargo bench --bench topological_sort_bench
//
// Save baseline
// cargo bench --bench topological_sort_bench -- --save-baseline before_optimization
//
// Compare with baseline
// cargo bench --bench topological_sort_bench -- --baseline before_optimization

fn benchmark_topological_sort(c: &mut Criterion) {
    let (nodes, edges) = generate_random_graph_data(20, 20, 3);
    let graph = build_graph(nodes, edges);

    c.bench_function("topological_sort", |b| {
        b.iter(|| {
            let sorting =
                topological_sort::<Node, Edge, u32, _>(&graph, Some(|n: &Node| (n.key() as u32)));
            black_box(sorting)
        })
    });
}

criterion_group!(benches, benchmark_topological_sort);
criterion_main!(benches);
