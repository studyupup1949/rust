# a-star

[![Crates.io](https://img.shields.io/crates/v/a-star.svg)](https://crates.io/crates/a-star)
[![Docs.rs](https://docs.rs/a-star/badge.svg)](https://docs.rs/a-star)
[![License: MIT](https://img.shields.io/crates/l/a-star.svg)](https://opensource.org/licenses/MIT)

This library provides a generic a-star search algorithm.

    a-star = "0.1.0-rc.1"

## Overview

A generic, reusable A* pathfinding implementation. The algorithm is parameterized over:

- **Node** -- the type of node in the graph
- **Cost** -- the type used for edge weights and heuristics
- **Graph** -- a trait providing neighbors and heuristic
- **DiscoveredSet** -- a trait abstracting the priority queue

## Core Types

- `Graph<Node, Cost>` -- trait for providing graph structure
- `AStar` -- the search algorithm, reusable across multiple searches
- `Path<Node, Cost>` -- the result of a search (nodes + total cost)
- `NodeCost<Node, Cost>` -- a node with an associated cost
- `DiscoveredSet` -- trait for the priority queue
- `MinHeap` -- binary min-heap implementation of `DiscoveredSet`

## Example

```rust
use a_star::*;

// Implement Graph for your domain.
struct MyGraph;

impl Graph<u32, u32> for MyGraph {
    fn neighbors(&self, node: u32, neighbors: &mut Vec<NodeCost<u32, u32>>) {
        // Add neighbors with their edge costs.
    }

    fn heuristic(&self, node: u32, goal: u32) -> u32 {
        // Return an admissible estimate.
        0
    }
}

let graph: MyGraph = MyGraph;
let discovered: MinHeap<u32, u32> = MinHeap::default ();
let mut search: AStar<u32, u32, MinHeap<u32, u32>, MyGraph> = AStar::new( & graph, discovered);

if let Some(path) = search.search(0, 10) {
println ! ("cost: {}", path.cost());
println ! ("nodes: {:?}", path.nodes());
}
```
