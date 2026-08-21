# Ade-graph

`ade-graph` provides the core graph data structures and utilities for directed graphs within the ADE ecosystem. It focuses on efficient representation and manipulation of graph data.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ade-graph = "0.1.0"
```

## Usage Example

The `ade-graph` crate offers utilities for building and working with directed graphs. The `build_graph` function is a convenient way to construct a graph from a list of nodes and edges.

```rust
use ade_graph::build::build_graph;
use ade_graph::GraphViewTrait;

fn main() {
    // Define nodes and edges for a simple directed graph.
    // Nodes: 0, 1, 2
    // Edges: 0 -> 1, 1 -> 2, 2 -> 1
    let nodes = vec![0, 1, 2];
    let edges = vec![(0, 1), (1, 2), (2, 1)];

    // Build the graph using the provided nodes and edges.
    let graph = build_graph(nodes, edges);
}
```

## Documentation

The complete documentation is available on [docs.rs](https://docs.rs/ade-graph).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
