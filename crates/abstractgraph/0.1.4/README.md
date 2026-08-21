abstractgraph - Graph algorithms on user supplied data structures
===

It's quite common for graph structures to be defined implicitly by
other data structures.  However, most graph algorithm libraries
require a specific input representation of the graph, meaning you need
to awkwardly convert your internal data structures into that
representation first.

`abstractgraph` lets you run graph algorithms directly on your own
data structures, by having them implement traits to expose the graph structure.

Currently implemented algorithms:
 * Depth First Search
 * Breadth First Search
 * Dijkstra's Algorithm (single source shortest path)

`abstractgraph` began as a port of the [`aga`][1] and [`agar`][2]
modules from ccan.

## Usage

Add the following to your `Cargo.toml`:
```toml
[dependencies]
abstractgraph = "0.1"
```

## License

This library is licensed under the LGPL, version 2.1 or later.

[1]: https://ccodearchive.net/info/aga.html
[2]: https://ccodearchive.net/info/agar.html
