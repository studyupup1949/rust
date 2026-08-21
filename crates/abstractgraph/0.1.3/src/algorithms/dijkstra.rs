//! Dijkstra's Single Source shortest path algorithm

use std::hash::Hash;
use std::collections;
use std::ops::Add;
use num;
use priority_queue;

use crate::DirectedGraph;
use crate::WeightedDirectedGraph;
use crate::OutboundEdge;
use crate::weight::WeightedOutboundEdge;

#[derive(Debug)]
struct NodeState<G, W>
where
    G: DirectedGraph+std::fmt::Debug,
    G::Node: std::fmt::Debug,
    G::Edge: std::fmt::Debug,
    W: std::fmt::Debug,
{
    complete: bool,
    distance: W,
    prev: Option<(G::Node, G::Edge)>,
}

impl<G, W> NodeState<G, W>
where
    G: DirectedGraph+std::fmt::Debug,
    G::Node: Clone+std::fmt::Debug,
    G::Edge: Clone+std::fmt::Debug,
    W: Clone+std::fmt::Debug,
{
    fn unwrap_distance(&self) -> W {
        if !self.complete {
            panic!();
        }
        self.distance.clone()
    }
    fn unwrap_prev(&self) -> Option<(G::Node, G::Edge)> {
        if !self.complete {
            panic!();
        }
        self.prev.clone()
    }
}

/// An in-progress Dijkstra's algorithm search
///
/// This implements `Iterator<G::Node>`, yielding graph nodes in
/// proximity order.  Any given node will be yielded at most once
/// (they will not be yielded at all if they are not reachable from a
/// provided start node).
pub struct Dijkstra<'a, G, W>
where
    G: WeightedDirectedGraph<W>+std::fmt::Debug,
    <G as DirectedGraph>::Node: Eq+Hash+std::fmt::Debug,
    <G as DirectedGraph>::Edge: Eq+Hash+std::fmt::Debug,
    W: Ord + std::ops::Add + std::fmt::Debug,
{
    graph: &'a G,
    pq: priority_queue::PriorityQueue<G::Node, std::cmp::Reverse<W>>,
    state: collections::HashMap<G::Node, NodeState<G, W>>,
}

impl<'a, G, W> Dijkstra<'a, G, W>
where
    G: 'a+WeightedDirectedGraph<W>+std::fmt::Debug,
    <G as DirectedGraph>::Node: Eq+Hash+Clone,
    <G as DirectedGraph>::Edge: Eq+Hash+Clone,
    <G as DirectedGraph>::Edge: WeightedOutboundEdge<G::Node, W>,
    for<'b> W: Add<&'b W, Output=W>,
    W: Ord+num::Zero+Clone,
    G::Node: std::fmt::Debug,
    G::Edge: std::fmt::Debug,
    W: std::fmt::Debug,
{
    /// Start a new single-source proximity first traversal
    ///
    /// As constructed, it will yield no nodes at all, since it has no
    /// starting node.  You need to use `search_from` to give it one
    /// or more places to start the search from.
    pub fn new(graph: &'a G) -> Dijkstra<'a, G, W> {
        Dijkstra {
            graph,
            pq: priority_queue::PriorityQueue::new(),
            state: collections::HashMap::new(),
        }
    }

    pub fn add_source(&mut self, node: <G as DirectedGraph>::Node) {
        let ns = NodeState {
            complete: false,
            distance: W::zero(),
            prev: None
        };
        self.state.insert(node.clone(), ns);
        self.pq.push(node, std::cmp::Reverse(W::zero()));
    }

    fn step(&mut self) -> Option<G::Node> {
        let (node, _) = self.pq.pop()?;
        let distance = self.state.get(&node).unwrap().distance.clone();

        dbg!(&node);
        dbg!(&distance);
        for e in self.graph.edges_from(node.clone()) {
            dbg!(&e);
            let dest = e.destination();
            dbg!(&dest);
            let cost = e.weight();
            dbg!(&cost);
            let dest_distance = cost + &distance;

            dbg!(&dest_distance);
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
            return Some(ns.unwrap_distance());
        }

        while let Some(n) = self.step() {
            if n == node {
                let d = self.state.get(&node).unwrap().unwrap_distance();
                          
                return Some(d);
            }
        }

        None
    }

    pub fn prev(&mut self, node: G::Node) -> Option<Option<(G::Node, G::Edge)>> {
        if let Some(ns) = self.state.get(&node) {
            return Some(ns.unwrap_prev());
        }

        while let Some(n) = self.step() {
            if n == node {
                let p = self.state.get(&node).unwrap().unwrap_prev();
                          
                return Some(p);
            }
        }

        None
    }
    
    fn candidate_path(&mut self, node: G::Node, distance: W,
                      prev: (G::Node, G::Edge)) {
        dbg!(&node);
        dbg!(&distance);
        dbg!(self.pq.get_priority(&node));
        match self.state.get_mut(&node) {
            None => {
                let ns = NodeState {
                    complete: false,
                    distance: distance.clone(),
                    prev: Some(prev),
                };
                self.pq.push(node.clone(), std::cmp::Reverse(distance));
                self.state.insert(node, ns);
            },
            Some(ns) => {
                dbg!(&ns);

                dbg!(&ns.distance);
                if distance < ns.distance {
                    assert!(!ns.complete);
                    self.pq.change_priority(&node, std::cmp::Reverse(distance));
                }
            },
        }
    }
}

impl<'a, G, W> Iterator for Dijkstra<'a, G, W>
where
    G: 'a+WeightedDirectedGraph<W>+std::fmt::Debug,
    <G as DirectedGraph>::Node: Eq+Hash+Clone,
    <G as DirectedGraph>::Edge: Eq+Hash+Clone,
    <G as DirectedGraph>::Edge: WeightedOutboundEdge<G::Node, W>,
    for<'b> W: Add<&'b W, Output=W>,
    W: Ord+num::Zero+Clone,
    G::Node: std::fmt::Debug,
    G::Edge: std::fmt::Debug,
    W: std::fmt::Debug,
{
    type Item = <G as DirectedGraph>::Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.step()
    }
}
