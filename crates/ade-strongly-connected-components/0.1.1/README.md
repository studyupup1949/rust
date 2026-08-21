# Ade-strongly-connected-components

D.J. Pearce's algorithm for finding strongly connected components (SCCs).
Information Processing Letters 116 (2016) 47-52

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ade-strongly-connected-components = "0.1.0"
```

## Usage Example

The `ade-strongly-connected-components` crate provides efficient algorithms for finding strongly connected components (SCCs) in directed graphs. Here's an example using the iterative implementation:

```rust
use ade_strongly_connected_components::scc_iterative;
use ade_graph::utils::build::build_graph;
use ade_graph::GraphViewTrait;

fn main() {
    // Define a graph with nodes 0, 1, 2, 3, 4 and specific edges.
    // This graph has three SCCs: {0, 1, 2}, {3}, {4}.
    let nodes = vec![0, 1, 2, 3, 4];
    let edges = vec![
        (0, 1), (1, 2), (2, 0), // First SCC: 0 -> 1 -> 2 -> 0
        (1, 3), // Edge from SCC {0, 1, 2} to node 3
        (3, 4), // Edge from node 3 to node 4
    ];

    let graph = build_graph(nodes, edges);

    // Find the strongly connected components using the iterative algorithm.
    let sccs = scc_iterative(&graph);

    // The result is a vector of vectors, where each inner vector represents an SCC.
    // The order of SCCs and nodes within an SCC may vary.
    println!("Strongly Connected Components: {:?}", sccs);
}
```

## Documentation

The complete documentation is available on [docs.rs](https://docs.rs/ade-strongly-connected-components).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Benchmark

```
# Save baseline
cargo bench --bench scc_bench -- --save-baseline before_optimization

# Compare with baseline
cargo bench --bench scc_bench -- --baseline before_optimization
```

## Iterative version (pseudo-code)

```
procedure SCC_Iterative(Graph G)
    rindex[0..n-1] ← 0
    root[0..n-1] ← false
    index ← 1
    c ← n - 1
    vS_front ← empty stack   // call stack (front)
    vS_back  ← empty stack   // component stack (back)
    iS       ← empty stack   // iterator stack

    for each vertex v in G do
        if rindex[v] = 0 then
            VISIT(v)

procedure VISIT(v)
    BEGIN_VISITING(v)
    while not EMPTY(vS_front) do
        VISIT_LOOP()

procedure VISIT_LOOP()
    v ← TOP_FRONT(vS_front)
    i ← TOP_FRONT(iS)
    successors ← edges(v)

    while i ≤ length(successors) do
        if i > 0 then
            FINISH_EDGE(v, successors[i-1])
        if i < length(successors) and BEGIN_EDGE(v, i) then
            return
        i ← i + 1
    FINISH_VISITING(v)

procedure BEGIN_VISITING(v)
    PUSH_FRONT(vS_front, v)
    PUSH_FRONT(iS, 0)
    root[v] ← true
    rindex[v] ← index
    index ← index + 1

procedure FINISH_VISITING(v)
    POP_FRONT(vS_front)
    POP_FRONT(iS)
    if root[v] = true then
        index ← index - 1
        while not EMPTY(vS_back) and rindex[v] ≤ rindex[TOP_BACK(vS_back)] do
            w ← POP_BACK(vS_back)
            rindex[w] ← c
            index ← index - 1
        rindex[v] ← c
        c ← c - 1
    else
        PUSH_BACK(vS_back, v)

procedure BEGIN_EDGE(v, k)
    w ← successors(v)[k]
    if rindex[w] = 0 then
        POP_FRONT(iS)
        PUSH_FRONT(iS, k + 1)
        BEGIN_VISITING(w)
        return true
    else
        return false

procedure FINISH_EDGE(v, w)
    if rindex[w] < rindex[v] then
        rindex[v] ← rindex[w]
        root[v] ← false
```
