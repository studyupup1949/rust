//! Graph algorithms
//!
//! This module provides implementations of classic graph algorithms:
//! - Dijkstra's Algorithm (shortest path)
//! - A* Search Algorithm (heuristic pathfinding)
//! - Bellman-Ford Algorithm (shortest path with negative weights)
//! - Floyd-Warshall Algorithm (all-pairs shortest path)

pub mod dijkstra;
pub mod astar;
pub mod bellman_ford;
pub mod floyd_warshall;

use std::collections::HashMap;

/// Represents a weighted, directed graph
#[derive(Debug, Clone)]
pub struct Graph {
    /// Adjacency list: node -> [(neighbor, weight), ...]
    pub edges: HashMap<usize, Vec<(usize, f64)>>,
    /// Number of nodes
    pub n_nodes: usize,
}

impl Graph {
    /// Creates a new empty graph
    pub fn new(n_nodes: usize) -> Self {
        Graph {
            edges: HashMap::new(),
            n_nodes,
        }
    }
    
    /// Adds a directed edge from u to v with the given weight
    pub fn add_edge(&mut self, u: usize, v: usize, weight: f64) {
        self.edges.entry(u).or_default().push((v, weight));
    }
    
    /// Adds an undirected edge (adds both directions)
    pub fn add_undirected_edge(&mut self, u: usize, v: usize, weight: f64) {
        self.add_edge(u, v, weight);
        self.add_edge(v, u, weight);
    }
    
    /// Returns neighbors of node u
    pub fn neighbors(&self, u: usize) -> &[(usize, f64)] {
        self.edges.get(&u).map(|v| v.as_slice()).unwrap_or(&[])
    }
}
