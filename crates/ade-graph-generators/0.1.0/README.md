# Ade-graph-generators

`ade-graph-generators` provides functions for generating complete and random graphs. These utilities are particularly useful for testing and benchmarking graph algorithms.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ade-graph-generators = "0.1.0"
```

## Usage Example

This example demonstrates how to use the `complete_graph_data` function to generate nodes and edges for a complete directed graph. The output is meant to be used as input for the `build_graph` function.

```rust
use ade_graph_generators::complete_graph_data;

fn main() {
    // Generate data for a complete graph with 4 nodes.
    // In a complete graph, every pair of distinct vertices is connected by a unique edge.
    let n = 4;
    let (nodes, edges) = complete_graph_data(n);

    // The nodes will be [0, 1, 2, 3].
    println!("Generated nodes: {:?}", nodes);
    assert_eq!(nodes, vec![0, 1, 2, 3]);

    // The edges will include all possible directed connections between distinct nodes.
    // For n=4, there will be n * (n - 1) = 4 * 3 = 12 edges.
    println!("Generated edges: {:?}", edges);
    assert_eq!(edges.len(), n * (n - 1));

    // Example check for one of the edges (e.g., from node 0 to node 1)
    assert!(edges.contains(&(0, 1)));
    // Example check for another edge
    assert!(edges.contains(&(1, 0)));
}
```

## Documentation

The complete documentation is available on [docs.rs](https://docs.rs/ade-graph-generators).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
