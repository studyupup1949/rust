//! A* Search Algorithm
//!
//! A* is an informed search algorithm that finds the shortest path using a heuristic
//! function to guide the search. It's widely used in pathfinding for games, robotics,
//! and navigation systems.
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::graph::{Graph, astar};
//!
//! let mut graph = Graph::new(4);
//! graph.add_edge(0, 1, 1.0);
//! graph.add_edge(1, 2, 1.0);
//! graph.add_edge(2, 3, 1.0);
//!
//! // Simple heuristic: assume all remaining distance is 1
//! let heuristic = |node: usize| if node == 3 { 0.0 } else { 1.0 };
//!
//! let result = astar::find_path(&graph, 0, 3, heuristic).unwrap();
//! assert_eq!(result.1.len(), 4); // Path has 4 nodes
//! ```

use crate::graph::Graph;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct State {
    node: usize,
    f_score: f64, // g_score + h_score
    g_score: f64, // actual cost from start
}

impl Eq for State {}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.f_score == other.f_score && self.node == other.node
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap based on f_score
        other.f_score.partial_cmp(&self.f_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Finds shortest path from start to goal using A* algorithm
///
/// # Arguments
///
/// * `graph` - The input graph
/// * `start` - Starting node
/// * `goal` - Goal node
/// * `heuristic` - Admissible heuristic function h(n) estimating cost from n to goal
///
/// # Returns
///
/// Option containing (cost, path) if a path exists
///
/// # Requirements
///
/// The heuristic must be admissible (never overestimate) and consistent for optimality
pub fn find_path<H>(
    graph: &Graph,
    start: usize,
    goal: usize,
    heuristic: H,
) -> Option<(f64, Vec<usize>)>
where
    H: Fn(usize) -> f64,
{
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<usize, usize> = HashMap::new();
    let mut g_score: HashMap<usize, f64> = HashMap::new();
    let mut closed_set: HashSet<usize> = HashSet::new();
    
    g_score.insert(start, 0.0);
    open_set.push(State {
        node: start,
        f_score: heuristic(start),
        g_score: 0.0,
    });
    
    while let Some(State { node, g_score: current_g, .. }) = open_set.pop() {
        // Check if already processed with better score
        if closed_set.contains(&node) {
            continue;
        }
        
        // Found goal
        if node == goal {
            return Some((current_g, reconstruct_path(&came_from, start, goal)));
        }
        
        closed_set.insert(node);
        
        // Explore neighbors
        for &(neighbor, weight) in graph.neighbors(node) {
            if closed_set.contains(&neighbor) {
                continue;
            }
            
            let tentative_g = current_g + weight;
            
            // Skip if we've found a better path to neighbor
            if let Some(&existing_g) = g_score.get(&neighbor) {
                if tentative_g >= existing_g {
                    continue;
                }
            }
            
            // This is the best path to neighbor so far
            came_from.insert(neighbor, node);
            g_score.insert(neighbor, tentative_g);
            
            let h = heuristic(neighbor);
            open_set.push(State {
                node: neighbor,
                f_score: tentative_g + h,
                g_score: tentative_g,
            });
        }
    }
    
    None // No path found
}

/// Reconstructs the path from start to goal
fn reconstruct_path(
    came_from: &HashMap<usize, usize>,
    start: usize,
    goal: usize,
) -> Vec<usize> {
    let mut path = vec![goal];
    let mut current = goal;
    
    while current != start {
        if let Some(&prev) = came_from.get(&current) {
            path.push(prev);
            current = prev;
        } else {
            break;
        }
    }
    
    path.reverse();
    path
}

/// A* algorithm with early termination and path weight limit
pub fn find_path_bounded<H>(
    graph: &Graph,
    start: usize,
    goal: usize,
    heuristic: H,
    max_cost: f64,
) -> Option<(f64, Vec<usize>)>
where
    H: Fn(usize) -> f64,
{
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<usize, usize> = HashMap::new();
    let mut g_score: HashMap<usize, f64> = HashMap::new();
    let mut closed_set: HashSet<usize> = HashSet::new();
    
    g_score.insert(start, 0.0);
    open_set.push(State {
        node: start,
        f_score: heuristic(start),
        g_score: 0.0,
    });
    
    while let Some(State { node, g_score: current_g, f_score }) = open_set.pop() {
        // Early termination if f_score exceeds limit
        if f_score > max_cost {
            return None;
        }
        
        if closed_set.contains(&node) {
            continue;
        }
        
        if node == goal {
            return Some((current_g, reconstruct_path(&came_from, start, goal)));
        }
        
        closed_set.insert(node);
        
        for &(neighbor, weight) in graph.neighbors(node) {
            if closed_set.contains(&neighbor) {
                continue;
            }
            
            let tentative_g = current_g + weight;
            
            // Skip if exceeds cost limit
            if tentative_g > max_cost {
                continue;
            }
            
            if let Some(&existing_g) = g_score.get(&neighbor) {
                if tentative_g >= existing_g {
                    continue;
                }
            }
            
            came_from.insert(neighbor, node);
            g_score.insert(neighbor, tentative_g);
            
            let h = heuristic(neighbor);
            open_set.push(State {
                node: neighbor,
                f_score: tentative_g + h,
                g_score: tentative_g,
            });
        }
    }
    
    None
}

/// Grid coordinates for 2D pathfinding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        GridPos { x, y }
    }
    
    /// Manhattan distance heuristic
    pub fn manhattan_distance(&self, other: &GridPos) -> f64 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as f64
    }
    
    /// Euclidean distance heuristic
    pub fn euclidean_distance(&self, other: &GridPos) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_path() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1, 1.0);
        graph.add_edge(1, 2, 1.0);
        graph.add_edge(2, 3, 1.0);
        graph.add_edge(0, 3, 5.0);
        
        // Zero heuristic (degrades to Dijkstra)
        let heuristic = |_: usize| 0.0;
        
        let result = find_path(&graph, 0, 3, heuristic).unwrap();
        
        assert_eq!(result.0, 3.0);
        assert_eq!(result.1, vec![0, 1, 2, 3]);
    }
    
    #[test]
    fn test_with_heuristic() {
        let mut graph = Graph::new(5);
        graph.add_edge(0, 1, 2.0);
        graph.add_edge(0, 2, 1.0);
        graph.add_edge(1, 3, 1.0);
        graph.add_edge(2, 3, 5.0);
        graph.add_edge(3, 4, 1.0);
        
        // Simple heuristic
        let heuristic = |node: usize| (4 - node) as f64;
        
        let result = find_path(&graph, 0, 4, heuristic).unwrap();
        
        assert_eq!(result.0, 4.0); // 0->1->3->4
    }
    
    #[test]
    fn test_no_path() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1, 1.0);
        
        let heuristic = |_: usize| 0.0;
        
        let result = find_path(&graph, 0, 2, heuristic);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_manhattan_distance() {
        let p1 = GridPos::new(0, 0);
        let p2 = GridPos::new(3, 4);
        
        assert_eq!(p1.manhattan_distance(&p2), 7.0);
        assert!((p1.euclidean_distance(&p2) - 5.0).abs() < 0.001);
    }
}
