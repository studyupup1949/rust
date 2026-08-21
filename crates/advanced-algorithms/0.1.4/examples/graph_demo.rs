//! Example demonstrating graph algorithms

use advanced_algorithms::graph::{Graph, dijkstra, astar, bellman_ford, floyd_warshall};

fn main() {
    println!("=== Graph Algorithms Demo ===\n");
    
    // Create a sample graph
    let mut graph = Graph::new(6);
    
    // Add edges (creating a small city network)
    graph.add_undirected_edge(0, 1, 7.0);  // City A to B
    graph.add_undirected_edge(0, 2, 9.0);  // City A to C
    graph.add_undirected_edge(0, 5, 14.0); // City A to F
    graph.add_undirected_edge(1, 2, 10.0); // City B to C
    graph.add_undirected_edge(1, 3, 15.0); // City B to D
    graph.add_undirected_edge(2, 3, 11.0); // City C to D
    graph.add_undirected_edge(2, 5, 2.0);  // City C to F
    graph.add_undirected_edge(3, 4, 6.0);  // City D to E
    graph.add_undirected_edge(4, 5, 9.0);  // City E to F
    
    // Dijkstra's Algorithm
    println!("=== Dijkstra's Algorithm ===");
    let start = 0;
    let goal = 4;
    
    let dijkstra_result = dijkstra::shortest_path(&graph, start);
    
    println!("Shortest path from {} to {}:", start, goal);
    if let Some(path) = dijkstra_result.path_to(goal) {
        println!("  Path: {:?}", path);
        println!("  Distance: {:.1}", dijkstra_result.distance[goal]);
    }
    
    println!("\nDistances from city {}:", start);
    for (city, &dist) in dijkstra_result.distance.iter().enumerate() {
        if dist.is_finite() {
            println!("  To city {}: {:.1}", city, dist);
        }
    }
    
    // A* Search Algorithm
    println!("\n=== A* Search Algorithm ===");
    
    // Simple heuristic: straight-line distance (simplified)
    let heuristic = |node: usize| {
        // Assume cities are arranged roughly in a line
        ((goal as i32 - node as i32).abs()) as f64 * 2.0
    };
    
    if let Some((cost, path)) = astar::find_path(&graph, start, goal, heuristic) {
        println!("A* path from {} to {}:", start, goal);
        println!("  Path: {:?}", path);
        println!("  Cost: {:.1}", cost);
    }
    
    // Floyd-Warshall Algorithm
    println!("\n=== Floyd-Warshall Algorithm ===");
    
    let fw_result = floyd_warshall::all_pairs_shortest_path(&graph).unwrap();
    
    println!("All-pairs shortest distances:");
    for i in 0..3 {
        for j in 0..3 {
            if i != j {
                let dist = fw_result.distance(i, j);
                if dist.is_finite() {
                    println!("  {} to {}: {:.1}", i, j, dist);
                }
            }
        }
    }
    
    // Graph properties
    if let Some(diam) = floyd_warshall::diameter(&graph) {
        println!("\nGraph diameter: {:.1}", diam);
    }
    
    if let Some(centers) = floyd_warshall::find_center(&graph) {
        println!("Graph center(s): {:?}", centers);
    }
    
    // Bellman-Ford with negative weights
    println!("\n=== Bellman-Ford Algorithm ===");
    
    let mut graph_negative = Graph::new(4);
    graph_negative.add_edge(0, 1, 4.0);
    graph_negative.add_edge(0, 2, 5.0);
    graph_negative.add_edge(1, 2, -2.0); // Negative edge
    graph_negative.add_edge(1, 3, 6.0);
    graph_negative.add_edge(2, 3, 3.0);
    
    match bellman_ford::shortest_path(&graph_negative, 0) {
        Ok(bf_result) => {
            println!("Bellman-Ford (with negative weights):");
            for (node, &dist) in bf_result.distance.iter().enumerate() {
                if dist.is_finite() {
                    println!("  Distance to {}: {:.1}", node, dist);
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    
    // Negative cycle detection
    let mut graph_cycle = Graph::new(3);
    graph_cycle.add_edge(0, 1, 1.0);
    graph_cycle.add_edge(1, 2, -3.0);
    graph_cycle.add_edge(2, 0, 1.0); // Creates negative cycle
    
    println!("\nNegative cycle detection:");
    if bellman_ford::has_negative_cycle(&graph_cycle) {
        println!("  Negative cycle detected!");
        if let Some(cycle) = bellman_ford::find_negative_cycle(&graph_cycle) {
            println!("  Cycle: {:?}", cycle);
        }
    }
}
