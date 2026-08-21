//! Bellman-Ford Algorithm
//!
//! Computes shortest paths from a source vertex to all other vertices in a weighted
//! graph. Unlike Dijkstra's algorithm, it can handle graphs with negative edge weights
//! and can detect negative cycles.
//!
//! Time complexity: O(VE)
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::graph::{Graph, bellman_ford};
//!
//! let mut graph = Graph::new(4);
//! graph.add_edge(0, 1, 1.0);
//! graph.add_edge(1, 2, -1.0);  // Negative weight OK
//! graph.add_edge(2, 3, 1.0);
//!
//! let result = bellman_ford::shortest_path(&graph, 0).unwrap();
//! assert_eq!(result.distance[3], 1.0);
//! ```

use crate::graph::Graph;
use crate::{AlgorithmError, Result};

/// Result of Bellman-Ford algorithm
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

/// Computes shortest paths using Bellman-Ford algorithm
///
/// # Arguments
///
/// * `graph` - The input graph (can have negative weights)
/// * `source` - Starting node
///
/// # Returns
///
/// ShortestPathResult or error if negative cycle detected
///
/// # Errors
///
/// Returns error if the graph contains a negative cycle reachable from source
pub fn shortest_path(graph: &Graph, source: usize) -> Result<ShortestPathResult> {
    let n = graph.n_nodes;
    
    let mut distance = vec![f64::INFINITY; n];
    let mut previous = vec![None; n];
    
    distance[source] = 0.0;
    
    // Relax all edges V-1 times
    for _ in 0..n - 1 {
        let mut changed = false;
        
        for u in 0..n {
            if distance[u].is_infinite() {
                continue;
            }
            
            for &(v, weight) in graph.neighbors(u) {
                let new_distance = distance[u] + weight;
                
                if new_distance < distance[v] {
                    distance[v] = new_distance;
                    previous[v] = Some(u);
                    changed = true;
                }
            }
        }
        
        // Early termination if no changes
        if !changed {
            break;
        }
    }
    
    // Check for negative cycles
    for u in 0..n {
        if distance[u].is_infinite() {
            continue;
        }
        
        for &(v, weight) in graph.neighbors(u) {
            if distance[u] + weight < distance[v] {
                return Err(AlgorithmError::InvalidInput(
                    "Graph contains a negative cycle".to_string()
                ));
            }
        }
    }
    
    Ok(ShortestPathResult {
        distance,
        previous,
        source,
    })
}

/// Detects if the graph contains any negative cycle
///
/// # Arguments
///
/// * `graph` - The input graph
///
/// # Returns
///
/// `true` if a negative cycle exists, `false` otherwise
pub fn has_negative_cycle(graph: &Graph) -> bool {
    let n = graph.n_nodes;
    let mut distance = vec![0.0; n];
    
    // Relax all edges V-1 times
    for _ in 0..n - 1 {
        for u in 0..n {
            for &(v, weight) in graph.neighbors(u) {
                let new_distance = distance[u] + weight;
                if new_distance < distance[v] {
                    distance[v] = new_distance;
                }
            }
        }
    }
    
    // Check for negative cycle
    for u in 0..n {
        for &(v, weight) in graph.neighbors(u) {
            if distance[u] + weight < distance[v] {
                return true;
            }
        }
    }
    
    false
}

/// Finds a negative cycle in the graph if one exists
///
/// # Returns
///
/// Option containing the nodes in the negative cycle
pub fn find_negative_cycle(graph: &Graph) -> Option<Vec<usize>> {
    let n = graph.n_nodes;
    let mut distance = vec![0.0; n];
    let mut previous = vec![None; n];
    
    // Relax all edges V times to detect and mark negative cycles
    let mut last_updated = None;
    
    for _iteration in 0..n {
        let mut changed = false;
        
        for u in 0..n {
            for &(v, weight) in graph.neighbors(u) {
                let new_distance = distance[u] + weight;
                
                if new_distance < distance[v] {
                    distance[v] = new_distance;
                    previous[v] = Some(u);
                    changed = true;
                    last_updated = Some(v);
                }
            }
        }
        
        if !changed {
            return None;
        }
    }
    
    // If we had updates in the V-th iteration, there's a negative cycle
    if let Some(mut node) = last_updated {
        // Trace back to find a node definitely in the cycle
        for _ in 0..n {
            node = previous[node]?;
        }
        
        // Reconstruct the cycle
        let mut cycle = vec![node];
        let mut current = previous[node]?;
        
        while current != node {
            cycle.push(current);
            current = previous[current]?;
        }
        
        cycle.reverse();
        return Some(cycle);
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_positive_weights() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, 2.0);
        graph.add_edge(2, 3, 3.0);
        
        let result = shortest_path(&graph, 0).unwrap();
        
        assert_eq!(result.distance[3], 6.0);
    }
    
    #[test]
    fn test_negative_weights() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, -3.0);
        graph.add_edge(2, 3, 1.0);
        graph.add_edge(0, 3, 5.0);
        
        let result = shortest_path(&graph, 0).unwrap();
        
        assert_eq!(result.distance[3], -1.0); // 0->1->2->3
    }
    
    #[test]
    fn test_negative_cycle() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, -3.0);
        graph.add_edge(2, 0, 1.0); // Creates negative cycle
        
        let result = shortest_path(&graph, 0);
        assert!(result.is_err());
        
        assert!(has_negative_cycle(&graph));
        
        let cycle = find_negative_cycle(&graph);
        assert!(cycle.is_some());
    }
    
    #[test]
    fn test_disconnected() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1, 1.0);
        
        let result = shortest_path(&graph, 0).unwrap();
        
        assert!(result.distance[2].is_infinite());
    }
}
