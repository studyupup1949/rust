//! Dijkstra's Algorithm
//!
//! Finds the shortest path from a source node to all other nodes in a weighted graph
//! with non-negative edge weights. One of the most famous algorithms in computer science.
//!
//! Time complexity: O((V + E) log V) with binary heap
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::graph::{Graph, dijkstra};
//!
//! let mut graph = Graph::new(5);
//! graph.add_edge(0, 1, 4.0);
//! graph.add_edge(0, 2, 1.0);
//! graph.add_edge(2, 1, 2.0);
//! graph.add_edge(1, 3, 1.0);
//! graph.add_edge(2, 3, 5.0);
//!
//! let result = dijkstra::shortest_path(&graph, 0);
//! assert_eq!(result.distance[3], 4.0); // 0->2->1->3
//! ```

use crate::graph::Graph;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Node in the priority queue
#[derive(Debug, Clone)]
struct State {
    node: usize,
    distance: f64,
}

impl Eq for State {}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance && self.node == other.node
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other.distance.partial_cmp(&self.distance)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Result of Dijkstra's algorithm
#[derive(Debug, Clone)]
pub struct ShortestPathResult {
    /// Distance from source to each node
    pub distance: Vec<f64>,
    /// Previous node in shortest path
    pub previous: Vec<Option<usize>>,
    /// Source node
    pub source: usize,
}

impl ShortestPathResult {
    /// Reconstructs the path from source to target
    pub fn path_to(&self, target: usize) -> Option<Vec<usize>> {
        if self.distance[target].is_infinite() {
            return None;
        }
        
        let mut path = Vec::new();
        let mut current = target;
        
        while current != self.source {
            path.push(current);
            current = self.previous[current]?;
        }
        
        path.push(self.source);
        path.reverse();
        
        Some(path)
    }
}

/// Computes shortest paths from source to all nodes using Dijkstra's algorithm
///
/// # Arguments
///
/// * `graph` - The input graph
/// * `source` - Starting node
///
/// # Returns
///
/// ShortestPathResult containing distances and predecessors
///
/// # Panics
///
/// Panics if the graph contains negative edge weights
pub fn shortest_path(graph: &Graph, source: usize) -> ShortestPathResult {
    let n = graph.n_nodes;
    
    let mut distance = vec![f64::INFINITY; n];
    let mut previous = vec![None; n];
    let mut heap = BinaryHeap::new();
    
    distance[source] = 0.0;
    heap.push(State {
        node: source,
        distance: 0.0,
    });
    
    while let Some(State { node, distance: dist }) = heap.pop() {
        // Skip if we've already found a better path
        if dist > distance[node] {
            continue;
        }
        
        // Explore neighbors
        for &(neighbor, weight) in graph.neighbors(node) {
            assert!(weight >= 0.0, "Dijkstra's algorithm requires non-negative weights");
            
            let new_distance = dist + weight;
            
            if new_distance < distance[neighbor] {
                distance[neighbor] = new_distance;
                previous[neighbor] = Some(node);
                
                heap.push(State {
                    node: neighbor,
                    distance: new_distance,
                });
            }
        }
    }
    
    ShortestPathResult {
        distance,
        previous,
        source,
    }
}

/// Finds shortest path from source to a specific target node
///
/// Returns early once target is reached for efficiency
pub fn shortest_path_to_target(
    graph: &Graph,
    source: usize,
    target: usize,
) -> Option<(f64, Vec<usize>)> {
    let n = graph.n_nodes;
    
    let mut distance = vec![f64::INFINITY; n];
    let mut previous = vec![None; n];
    let mut heap = BinaryHeap::new();
    
    distance[source] = 0.0;
    heap.push(State {
        node: source,
        distance: 0.0,
    });
    
    while let Some(State { node, distance: dist }) = heap.pop() {
        // Found target
        if node == target {
            let result = ShortestPathResult {
                distance,
                previous,
                source,
            };
            
            return Some((dist, result.path_to(target)?));
        }
        
        if dist > distance[node] {
            continue;
        }
        
        for &(neighbor, weight) in graph.neighbors(node) {
            let new_distance = dist + weight;
            
            if new_distance < distance[neighbor] {
                distance[neighbor] = new_distance;
                previous[neighbor] = Some(node);
                
                heap.push(State {
                    node: neighbor,
                    distance: new_distance,
                });
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_path() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, 2.0);
        graph.add_edge(2, 3, 3.0);
        graph.add_edge(0, 3, 10.0);
        
        let result = shortest_path(&graph, 0);
        
        assert_eq!(result.distance[3], 6.0);
        
        let path = result.path_to(3).unwrap();
        assert_eq!(path, vec![0, 1, 2, 3]);
    }
    
    #[test]
    fn test_multiple_paths() {
        let mut graph = Graph::new(5);
        graph.add_edge(0, 1, 4.0);
        graph.add_edge(0, 2, 1.0);
        graph.add_edge(2, 1, 2.0);
        graph.add_edge(1, 3, 1.0);
        graph.add_edge(2, 3, 5.0);
        graph.add_edge(3, 4, 1.0);
        
        let result = shortest_path(&graph, 0);
        
        assert_eq!(result.distance[1], 3.0); // via 2
        assert_eq!(result.distance[3], 4.0); // via 2->1
        assert_eq!(result.distance[4], 5.0);
    }
    
    #[test]
    fn test_disconnected() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1, 1.0);
        
        let result = shortest_path(&graph, 0);
        
        assert!(result.distance[2].is_infinite());
        assert!(result.path_to(2).is_none());
    }
}
