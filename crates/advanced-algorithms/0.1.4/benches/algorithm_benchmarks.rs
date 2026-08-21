use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use advanced_algorithms::numerical::fft;
use advanced_algorithms::number_theory::miller_rabin;
use advanced_algorithms::number_theory::extended_euclidean;
use advanced_algorithms::optimization::monte_carlo;
use advanced_algorithms::graph::{Graph, dijkstra, bellman_ford, floyd_warshall};

fn fft_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("FFT");
    
    for size in [64, 128, 256, 512, 1024].iter() {
        let input: Vec<f64> = (0..*size)
            .map(|i| (i as f64).sin())
            .collect();
        
        group.bench_with_input(BenchmarkId::new("size", size), size, |b, _| {
            b.iter(|| fft::fft(black_box(&input)))
        });
    }
    group.finish();
}

fn number_theory_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Number Theory");
    
    group.bench_function("miller_rabin_large", |b| {
        b.iter(|| miller_rabin::is_prime(black_box(982451653), black_box(10)))
    });
    
    group.bench_function("extended_euclidean", |b| {
        b.iter(|| extended_euclidean::extended_gcd(black_box(123456789), black_box(987654321)))
    });
    
    group.finish();
}

fn optimization_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Optimization");
    
    // Monte Carlo integration for estimating pi
    let bounds = vec![(0.0, 1.0), (0.0, 1.0)];
    group.bench_function("monte_carlo_pi", |b| {
        b.iter(|| {
            monte_carlo::integrate(
                black_box(|x: &[f64]| if x[0]*x[0] + x[1]*x[1] <= 1.0 { 1.0 } else { 0.0 }),
                black_box(&bounds),
                black_box(10000),
            )
        })
    });
    
    group.finish();
}

fn graph_algorithms_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Graph Algorithms");
    
    // Create a sample graph
    let mut graph = Graph::new(6);
    graph.add_edge(0, 1, 4.0);
    graph.add_edge(0, 2, 1.0);
    graph.add_edge(1, 2, 2.0);
    graph.add_edge(1, 3, 5.0);
    graph.add_edge(2, 3, 8.0);
    graph.add_edge(2, 4, 3.0);
    graph.add_edge(3, 4, 1.0);
    graph.add_edge(3, 5, 2.0);
    graph.add_edge(4, 5, 6.0);
    
    group.bench_function("dijkstra_shortest_path", |b| {
        b.iter(|| dijkstra::shortest_path(black_box(&graph), black_box(0)))
    });
    
    group.bench_function("bellman_ford", |b| {
        b.iter(|| bellman_ford::shortest_path(black_box(&graph), black_box(0)))
    });
    
    group.bench_function("floyd_warshall", |b| {
        b.iter(|| floyd_warshall::all_pairs_shortest_path(black_box(&graph)))
    });
    
    group.finish();
}

criterion_group!(
    benches,
    fft_benchmarks,
    number_theory_benchmarks,
    optimization_benchmarks,
    graph_algorithms_benchmarks
);
criterion_main!(benches);
