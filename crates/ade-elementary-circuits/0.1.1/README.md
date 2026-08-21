# Ade-elementary-circuits

Johnson's algorithm for finding all elementary circuits in a directed graph
SIAM J. Comput., Vol. 4, No. 1, March 1975
https://www.cs.tufts.edu/comp/150GA/homeworks/hw1/Johnson%2075.PDF

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ade-elementary-circuits = "0.1.0"
```

## Usage Example

This example demonstrates how to use the `elementary_circuits` function to find all elementary circuits in a directed graph.

```rust
use ade_elementary_circuits::elementary_circuits;
use ade_graph::utils::build::build_graph;

fn main() {
    // Define a graph with nodes 0, 1, 2 and edges (0, 1), (1, 2), (2, 1).
    // This graph has one elementary circuit: 1 -> 2 -> 1.
    let graph = build_graph(vec![0, 1, 2], vec![(0, 1), (1, 2), (2, 1)]);

    // Find all elementary circuits in the graph.
    let circuits = elementary_circuits(&graph);

    // The expected result is a vector containing one circuit: [1, 2, 1].
    let expected = vec![vec![1, 2, 1]];

    // Print the found circuits.
    println!("Found circuits: {:?}", circuits);
    assert_eq!(circuits, expected);
}
```

## Documentation

The complete documentation is available on [docs.rs](https://docs.rs/ade-elementary-circuits).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Benchmark

```
# Run benchmarks within a crate
cargo bench --bench elementary_circuits_bench --features="test-utils"

# Save baseline
cargo bench --bench elementary_circuits_bench --features="test-utils" -- --save-baseline before_optimization

# Compare with baseline
cargo bench --bench elementary_circuits_bench --features="test-utils" -- --baseline before_optimization
```
