//! Floyd-Warshall Algorithm
//!
//! Computes shortest paths between all pairs of vertices in a weighted graph.
//! Can handle negative edge weights and detect negative cycles.
//!
//! Time complexity: O(V³)
//! Space complexity: O(V²)
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::graph::{Graph, floyd_warshall};
//!
//! let mut graph = Graph::new(4);
//! graph.add_edge(0, 1, 3.0);
//! graph.add_edge(1, 2, 1.0);
//! graph.add_edge(2, 3, 2.0);
//! graph.add_edge(0, 3, 10.0);
//!
//! let result = floyd_warshall::all_pairs_shortest_path(&graph).unwrap();
//! assert_eq!(result.distance(0, 3), 6.0);
//! ```

use crate::graph::Graph;
use crate::{AlgorithmError, Result};

/// Result of Floyd-Warshall algorithm
#[derive(Debug, Clone)]
pub struct AllPairsShortestPath {
    /// Distance matrix: distance[i][j] is shortest path from i to j
    distance: Vec<Vec<f64>>,
    /// Next node matrix for path reconstruction
    next: Vec<Vec<Option<usize>>>,
    /// Number of nodes
    _n: usize,
}

impl AllPairsShortestPath {
    /// Gets the shortest distance from u to v
    pub fn distance(&self, u: usize, v: usize) -> f64 {
        self.distance[u][v]
    }
    
    /// Reconstructs the shortest path from u to v
    pub fn path(&self, u: usize, v: usize) -> Option<Vec<usize>> {
        if self.distance[u][v].is_infinite() {
            return None;
        }
        
        let mut path = vec![u];
        let mut current = u;
        
        while current != v {
            current = self.next[current][v]?;
            path.push(current);
        }
        
        Some(path)
    }
    
    /// Returns the distance matrix
    pub fn distance_matrix(&self) -> &[Vec<f64>] {
        &self.distance
    }
}

/// Computes all-pairs shortest paths using Floyd-Warshall algorithm
///
/// # Arguments
///
/// * `graph` - The input graph
///
/// # Returns
///
/// AllPairsShortestPath result or error if negative cycle exists
///
/// # Errors
///
/// Returns error if the graph contains a negative cycle
pub fn all_pairs_shortest_path(graph: &Graph) -> Result<AllPairsShortestPath> {
    let n = graph.n_nodes;
    
    // Initialize distance matrix
    let mut distance = vec![vec![f64::INFINITY; n]; n];
    let mut next = vec![vec![None; n]; n];
    
    // Distance from a node to itself is 0
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        distance[i][i] = 0.0;
    }
    
    // Initialize with direct edges
    for u in 0..n {
        for &(v, weight) in graph.neighbors(u) {
            distance[u][v] = weight;
            next[u][v] = Some(v);
        }
    }
    
    // Floyd-Warshall main loop
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                let new_distance = distance[i][k] + distance[k][j];
                
                if new_distance < distance[i][j] {
                    distance[i][j] = new_distance;
                    next[i][j] = next[i][k];
                }
            }
        }
    }
    
    // Check for negative cycles
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        if distance[i][i] < 0.0 {
            return Err(AlgorithmError::InvalidInput(
                "Graph contains a negative cycle".to_string()
            ));
        }
    }
    
    Ok(AllPairsShortestPath {
        distance,
        next,
        _n: n,
    })
}

/// Computes the transitive closure of a graph
///
/// Returns a matrix where result[i][j] is true if there's a path from i to j
pub fn transitive_closure(graph: &Graph) -> Vec<Vec<bool>> {
    let n = graph.n_nodes;
    let mut reachable = vec![vec![false; n]; n];
    
    // A node can reach itself
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        reachable[i][i] = true;
    }
    
    // Initialize with direct edges
    #[allow(clippy::needless_range_loop)]
    for u in 0..n {
        for &(v, _) in graph.neighbors(u) {
            reachable[u][v] = true;
        }
    }
    
    // Warshall's algorithm for transitive closure
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                reachable[i][j] = reachable[i][j] || (reachable[i][k] && reachable[k][j]);
            }
        }
    }
    
    reachable
}

/// Finds the graph diameter (longest shortest path)
///
/// # Returns
///
/// The diameter, or None if the graph is not connected
pub fn diameter(graph: &Graph) -> Option<f64> {
    match all_pairs_shortest_path(graph) {
        Ok(apsp) => {
            let mut max_distance: f64 = 0.0;
            
            for i in 0..graph.n_nodes {
                for j in 0..graph.n_nodes {
                    let d = apsp.distance(i, j);
                    if d.is_finite() {
                        max_distance = max_distance.max(d);
                    } else if i != j {
                        // Graph is not connected
                        return None;
                    }
                }
            }
            
            Some(max_distance)
        }
        Err(_) => None,
    }
}

/// Finds the center of the graph (node with minimum eccentricity)
///
/// Eccentricity of a node is the maximum distance to any other node
pub fn find_center(graph: &Graph) -> Option<Vec<usize>> {
    let apsp = all_pairs_shortest_path(graph).ok()?;
    let n = graph.n_nodes;
    
    let mut eccentricities = Vec::with_capacity(n);
    
    for i in 0..n {
        let mut max_distance: f64 = 0.0;
        
        for j in 0..n {
            let d = apsp.distance(i, j);
            if d.is_finite() {
                max_distance = max_distance.max(d);
            } else {
                // Graph is not connected
                return None;
            }
        }
        
        eccentricities.push(max_distance);
    }
    
    let min_eccentricity = eccentricities.iter()
        .fold(f64::INFINITY, |a, &b| a.min(b));
    
    let centers: Vec<usize> = eccentricities.iter()
        .enumerate()
        .filter(|(_, &e)| (e - min_eccentricity).abs() < 1e-10)
        .map(|(i, _)| i)
        .collect();
    
    Some(centers)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_graph() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, 2.0);
        graph.add_edge(2, 3, 3.0);
        graph.add_edge(0, 3, 10.0);
        
        let result = all_pairs_shortest_path(&graph).unwrap();
        
        assert_eq!(result.distance(0, 3), 6.0);
        assert_eq!(result.distance(0, 2), 3.0);
        
        let path = result.path(0, 3).unwrap();
        assert_eq!(path, vec![0, 1, 2, 3]);
    }
    
    #[test]
    fn test_all_pairs() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, 2.0);
        graph.add_edge(0, 2, 5.0);
        
        let result = all_pairs_shortest_path(&graph).unwrap();
        
        assert_eq!(result.distance(0, 1), 1.0);
        assert_eq!(result.distance(0, 2), 3.0);
        assert_eq!(result.distance(1, 2), 2.0);
    }
    
    #[test]
    fn test_negative_cycle() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, -3.0);
        graph.add_edge(2, 0, 1.0);
        
        let result = all_pairs_shortest_path(&graph);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_transitive_closure() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, 1.0);
        graph.add_edge(2, 3, 1.0);
        
        let closure = transitive_closure(&graph);
        
        assert!(closure[0][3]); // 0 can reach 3
        assert!(!closure[3][0]); // 3 cannot reach 0
    }
    
    #[test]
    fn test_diameter() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, 1.0);
        graph.add_edge(2, 3, 1.0);
        graph.add_undirected_edge(0, 3, 1.0);
        
        let d = diameter(&graph);
        assert!(d.is_some());
        // Diameter is 3.0: longest path is 1→2→3→0 or 3→0→1→2
        assert_eq!(d.unwrap(), 3.0);
    }
}
