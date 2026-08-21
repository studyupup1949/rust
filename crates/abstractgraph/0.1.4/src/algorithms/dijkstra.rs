//! Dijkstra's Single Source shortest path algorithm

use std::collections;
use std::hash::Hash;

use priority_queue;

use crate::DirectedGraph;
use crate::EdgeWeight;
use crate::OutboundEdge;
use crate::WeightedOutboundEdge;

#[derive(Debug)]
struct NodeState<G: DirectedGraph, W: EdgeWeight> {
    complete: bool,
    distance: W,
    prev: Option<(G::Node, G::Edge)>,
}

/// An in-progress Dijkstra's algorithm search
///
/// This implements `Iterator<G::Node>`, yielding graph nodes in
/// proximity order.  Any given node will be yielded at most once
/// (they will not be yielded at all if they are not reachable from a
/// provided start node).
pub struct Dijkstra<'a, G, W>
where
    G: DirectedGraph,
    G::Node: Eq + Hash,
    G::Edge: Eq + Hash,
    W: EdgeWeight,
{
    graph: &'a G,
    pq: priority_queue::PriorityQueue<G::Node, std::cmp::Reverse<W>>,
    state: collections::HashMap<G::Node, NodeState<G, W>>,
}

impl<'a, G, W> Dijkstra<'a, G, W>
where
    G: 'a + DirectedGraph,
    G::Node: Eq + Hash + Clone,
    G::Edge: Eq + Hash + Clone,
    G::Edge: WeightedOutboundEdge<G::Node, W>,
    W: EdgeWeight,
{
    /// Start finding shortest paths from `start` in `graph`
    ///
    /// Returns a `Dijkstra` struct representing the in-progress
    /// algorithm finding shortest paths from `start` to all other
    /// reachable nodes.
    pub fn new(graph: &'a G, start: G::Node) -> Dijkstra<'a, G, W> {
        let mut d = Dijkstra {
            graph,
            pq: priority_queue::PriorityQueue::new(),
            state: collections::HashMap::new(),
        };
        let ns = NodeState {
            complete: false,
            distance: W::zero(),
            prev: None,
        };
        d.state.insert(start.clone(), ns);
        d.pq.push(start, std::cmp::Reverse(W::zero()));
        d
    }

    fn step(&mut self) -> Option<G::Node> {
        let (node, _) = self.pq.pop()?;
        let distance = self.state[&node].distance.clone();

        for e in self.graph.edges_from(&node) {
            let dest = e.destination();
            let cost = e.weight();
            let dest_distance = cost + &distance;

            if dest_distance < distance {
                panic!("Encountered negative edge cost");
            }
            self.candidate_path(dest, dest_distance, (node.clone(), e));
        }

        let ns = self.state.get_mut(&node).unwrap();

        ns.complete = true;

        Some(node)
    }

    pub fn distance(&mut self, node: G::Node) -> Option<W> {
        if let Some(ns) = self.state.get(&node) {
            return Some(ns.distance.clone());
        }

        while let Some(n) = self.step() {
            if n == node {
                return Some(self.state[&node].distance.clone());
            }
        }

        None
    }

    pub fn prev(&mut self, node: G::Node) -> Option<Option<(G::Node, G::Edge)>> {
        if let Some(ns) = self.state.get(&node) {
            return Some(ns.prev.clone());
        }

        while let Some(n) = self.step() {
            if n == node {
                return Some(self.state[&node].prev.clone());
            }
        }

        None
    }

    fn candidate_path(&mut self, node: G::Node, distance: W, prev: (G::Node, G::Edge)) {
        match self.state.get_mut(&node) {
            None => {
                let ns = NodeState {
                    complete: false,
                    distance: distance.clone(),
                    prev: Some(prev),
                };
                self.pq.push(node.clone(), std::cmp::Reverse(distance));
                self.state.insert(node, ns);
            }
            Some(ns) => {
                if distance < ns.distance {
                    assert!(!ns.complete);
                    self.pq.change_priority(&node, std::cmp::Reverse(distance));
                }
            }
        }
    }
}

impl<'a, G, W> Iterator for Dijkstra<'a, G, W>
where
    G: 'a + DirectedGraph,
    G::Node: Eq + Hash + Clone,
    G::Edge: Eq + Hash + Clone,
    G::Edge: WeightedOutboundEdge<G::Node, W>,
    W: EdgeWeight,
{
    type Item = G::Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.step()
    }
}
