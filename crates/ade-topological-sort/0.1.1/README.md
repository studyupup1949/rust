# Ade-topological-sort

`ade-topological-sort` provides a topological sorting algorithm for directed graphs. It can handle graphs with cycles (by returning an error) and allows for custom sorting of nodes based on provided keys.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ade-topological-sort = "0.1.0"
```

## Usage Example

The `topological_sort` function can sort the nodes of a directed graph such that for every directed edge from node `u` to node `v`, `u` comes before `v` in the ordering. It also supports sorting based on a custom key function, allowing for deterministic output when multiple valid topological sorts exist.

This example demonstrates sorting a graph where nodes are ordered by their keys in descending order.

```rust
use ade_topological_sort::topological_sort;
use ade_graph::utils::build::build_graph;
use ade_graph::GraphViewTrait;
use ade_graph::implementations::Node;

fn main() {
    // Define a graph with nodes 0, 1, 2 and edges 0 -> 1, 0 -> 2.
    // A valid topological sort could be [0, 1, 2] or [0, 2, 1].
    let graph = build_graph(vec![0, 1, 2], vec![(0, 1), (0, 2)]);

    // Sort topologically, using the node's key in descending order.
    // This ensures a deterministic output: [0, 2, 1].
    let sorted_nodes = topological_sort::<Node, _, u32, _>(&graph, Some(|n: &Node| -(n.key() as i32)))
        .expect("Topological sort failed");

    println!("Topologically sorted nodes (descending key): {:?}", sorted_nodes);
    assert_eq!(sorted_nodes, vec![0, 2, 1]);
}
```

## Documentation

The complete documentation is available on [docs.rs](https://docs.rs/ade-topological-sort).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Benchmark

```
# Run topological_sort_bench benchmark
cargo bench --bench topological_sort_bench

# Save baseline
cargo bench --bench topological_sort_bench -- --save-baseline before_optimization

# Compare with baseline
cargo bench --bench topological_sort_bench -- --baseline before_optimization
```
