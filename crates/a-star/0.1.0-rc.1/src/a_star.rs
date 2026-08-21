use crate::path::Path;
use crate::{DiscoveredSet, Graph, NodeCost};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::Add;

/// Responsible for performing searches using the `A*` algorithm.
pub struct AStar<'a, Node, Cost, DS, G>
where
    Node: Copy + Eq + Hash,
    Cost: Copy + Ord + Default + Add<Cost, Output = Cost>,
    DS: DiscoveredSet<Node, Cost>,
    G: Graph<Node, Cost>,
{
    graph: &'a G,
    discovered: DS,
    closed: HashSet<Node>,
    previous: HashMap<Node, Node>,
    g_scores: HashMap<Node, Cost>,
    neighbors: Vec<NodeCost<Node, Cost>>,
    path: Path<Node, Cost>,
}

impl<'a, Node, Cost, DS, G> AStar<'a, Node, Cost, DS, G>
where
    Node: Copy + Eq + Hash,
    Cost: Copy + Ord + Default + Add<Cost, Output = Cost>,
    DS: DiscoveredSet<Node, Cost>,
    G: Graph<Node, Cost>,
{
    //! Construction

    /// Creates a new [AStar] instance.
    pub fn new(graph: &'a G, discovered: DS) -> Self {
        Self {
            graph,
            discovered,
            closed: HashSet::new(),
            previous: HashMap::new(),
            g_scores: HashMap::new(),
            neighbors: Vec::new(),
            path: Path::new(0),
        }
    }
}

impl<'a, Node, Cost, DS, G> AStar<'a, Node, Cost, DS, G>
where
    Node: Copy + Eq + Hash,
    Cost: Copy + Ord + Default + Add<Cost, Output = Cost>,
    DS: DiscoveredSet<Node, Cost>,
    G: Graph<Node, Cost>,
{
    //! Search

    /// Finds the shortest path from the `start` node to the `goal` node, inclusive.
    pub fn search(&mut self, start: Node, goal: Node) -> Option<&Path<Node, Cost>> {
        self.reset();

        let h_score: Cost = self.graph.heuristic(start, goal);
        self.discovered.push(NodeCost::new(start, h_score));
        self.g_scores.insert(start, Cost::default());

        while let Some(current) = self.next() {
            if current == goal {
                return Some(self.reconstruct_path(current));
            }

            // Skip already-expanded nodes.
            if !self.closed.insert(current) {
                continue;
            }

            let current_g_score: Cost = self
                .g_scores
                .get(&current)
                .copied()
                .expect("invalid impl: there must be a g-score for discovered nodes");

            self.neighbors.clear();
            self.graph.neighbors(current, &mut self.neighbors);
            for neighbor in &self.neighbors {
                let (neighbor, edge_cost): (Node, Cost) = (neighbor.node, neighbor.cost);
                let g_score: Cost = current_g_score + edge_cost;
                if self.is_best_g_score(neighbor, g_score) {
                    self.previous.insert(neighbor, current);
                    self.g_scores.insert(neighbor, g_score);

                    let h_score: Cost = self.graph.heuristic(neighbor, goal);
                    let f_score: Cost = g_score + h_score;
                    self.discovered.push(NodeCost::new(neighbor, f_score));
                }
            }
        }

        None
    }

    /// Resets the state for another iteration of the `A*` algorithm.
    fn reset(&mut self) {
        self.discovered.clear();
        self.closed.clear();
        self.previous.clear();
        self.g_scores.clear();
    }

    /// Gets the next node to process.
    fn next(&mut self) -> Option<Node> {
        self.discovered
            .pop()
            .map(|node_cost: NodeCost<Node, Cost>| node_cost.node)
    }

    /// Checks if the `g_score` is the best `g_score` known for the `node`.
    fn is_best_g_score(&self, node: Node, g_score: Cost) -> bool {
        if let Some(current_g_score) = self.g_scores.get(&node) {
            g_score < *current_g_score
        } else {
            true
        }
    }

    /// Reconstructs the path from `start` to `last`.
    fn reconstruct_path(&mut self, mut last: Node) -> &Path<Node, Cost> {
        self.path.clear();
        self.path
            .set_cost(self.g_scores.get(&last).copied().unwrap_or(Cost::default()));
        self.path.push(last);
        while let Some(current) = self.previous.get(&last) {
            self.path.push(*current);
            last = *current;
        }
        &self.path
    }
}
