//! Depth First Search

use std::hash::Hash;
use std::collections;

use crate::DirectedGraph;
use crate::Neighbors;

struct NodeState<G: DirectedGraph> {
    node: G::Node,
    neighbors: Neighbors<G>,
}

impl<'a, G: DirectedGraph+'a> NodeState<G> {
    fn new(g: &'a G, node: <G as DirectedGraph>::Node) -> Self {
        NodeState {
            node: node.clone(), neighbors: g.neighbors(node),
        }
    }
}

/// An in-progress depth-first search
///
/// This implements `Iterator<G::Node>`, yielding graph nodes in
/// depth-first search order.  Any given node will be yielded at
/// most once (they will not be yielded at all if they are not
/// reachable from a provided start node).
pub struct DepthFirstSearch<'a, G>
where
    G: DirectedGraph,
    <G as DirectedGraph>::Node: Eq+Hash,
{
    graph: &'a G,
    visited: collections::HashSet<<G as DirectedGraph>::Node>,
    stack: Vec<NodeState<G>>,
}

impl<'a, G> DepthFirstSearch<'a, G>
where
    G: 'a+DirectedGraph,
    <G as DirectedGraph>::Node: Eq+Hash,
{
    /// Start a new depth-first traversal
    ///
    /// As constructed, it will yield no nodes at all, since it has no
    /// starting node.  You need to use `search_from` to give it one
    /// or more places to start the search from.
    pub fn new(graph: &'a G) -> DepthFirstSearch<'a, G> {
        DepthFirstSearch {
            graph,
            visited: collections::HashSet::new(),
            stack: Vec::new(),
        }
    }

    /// "Seed" the search with a starting node
    ///
    /// Use `node` as a starting point for the search.
    ///
    ///  * If used before `next()` is first called, gives an initial
    ///  starting point for the search - the search will yield `node`
    ///  then all nodes reachable from `node` nodes in depth-first
    ///  order.
    ///
    ///  * If used after `next()` returns `None`, continues the search
    ///    from `node`.  If node was previously visited, this will
    ///    have no effect, otherwise `node` will be yielded followed
    ///    by any nodes reachable from `node` but not previously
    ///    yielded, in depth-first order.
    ///
    ///  * If used while still yielding nodes from a previous starting
    ///  point will give consistent, but possibly confusing results.
    pub fn search_from(&mut self, node: <G as DirectedGraph>::Node) {
        self.stack.push(NodeState::new(self.graph, node));
    }
}

impl<'a, G> Iterator for DepthFirstSearch<'a, G>
where
    G: 'a+DirectedGraph,
    <G as DirectedGraph>::Node: Eq+Hash,
{
    type Item = <G as DirectedGraph>::Node;

    fn next(&mut self) -> Option<Self::Item> {
        let next = 'outer: loop {
            if self.stack.is_empty() {
                return None;
            }

            {
                let state = &mut self.stack.last_mut().unwrap();

                if ! self.visited.contains(&state.node) {
                    self.visited.insert(state.node.clone());
                    return Some(state.node.clone());
                }

                while let Some(to) = state.neighbors.next() {
                    if ! self.visited.contains(&to) {
                        break 'outer to;
                    }
                }
            }

            self.stack.pop();
        };

        self.visited.insert(next.clone());
        self.stack.push(NodeState::new(self.graph, next.clone()));
        Some(next)
    }
}
