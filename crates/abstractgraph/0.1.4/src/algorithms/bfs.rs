//! Breadth First Search

use std::collections;
use std::hash::Hash;

use crate::DirectedGraph;
use crate::Neighbors;

struct NodeState<G: DirectedGraph> {
    node: G::Node,
    neighbors: Neighbors<G>,
}

impl<'a, G: DirectedGraph + 'a> NodeState<G> {
    fn new(g: &'a G, node: G::Node) -> Self {
        NodeState {
            node: node.clone(),
            neighbors: g.neighbors(&node),
        }
    }
}

/// An in-progress breadth-first search
///
/// This implements `Iterator<G::Node>`, yielding graph nodes in
/// breadth-first search order.  Any given node will be yielded at
/// most once (they will not be yielded at all if they are not
/// reachable from a provided start node).
pub struct BreadthFirstSearch<'a, G>
where
    G: DirectedGraph,
    G::Node: Eq + Hash,
{
    graph: &'a G,
    visited: collections::HashSet<G::Node>,
    queue: collections::VecDeque<NodeState<G>>,
}

impl<'a, G> BreadthFirstSearch<'a, G>
where
    G: 'a + DirectedGraph,
    G::Node: Eq + Hash,
{
    /// Start a new breadth-first traversal
    ///
    /// As constructed, it will yield no nodes at all, since it has no
    /// starting node.  You need to use `search_from` to give it one
    /// or more places to start the search from.
    pub fn new(graph: &'a G) -> BreadthFirstSearch<'a, G> {
        BreadthFirstSearch {
            graph,
            visited: collections::HashSet::new(),
            queue: collections::VecDeque::new(),
        }
    }

    /// "Seed" the search with a starting node
    ///
    /// Use `node` as a starting point for the search.
    ///
    ///  * If used before `next()` is first called, gives an initial
    ///  starting point for the search - the search will yield `node`
    ///  then all nodes reachable from `node` nodes in breadth-first
    ///  order.
    ///
    ///  * If used after `next()` returns `None`, continues the search
    ///    from `node`.  If node was previously visited, this will
    ///    have no effect, otherwise `node` will be yielded followed
    ///    by any nodes reachable from `node` but not previously
    ///    yielded, in breadth-first order.
    ///
    ///  * If used while still yielding nodes from a previous starting
    ///  point will give consistent, but possibly confusing results.
    pub fn search_from(&mut self, node: G::Node) {
        self.queue.push_back(NodeState::new(self.graph, node));
    }
}

impl<'a, G> Iterator for BreadthFirstSearch<'a, G>
where
    G: 'a + DirectedGraph,
    G::Node: Eq + Hash,
{
    type Item = G::Node;

    fn next(&mut self) -> Option<Self::Item> {
        let next = 'outer: loop {
            if self.queue.is_empty() {
                return None;
            }

            {
                let state = &mut self.queue.front_mut().unwrap();

                if !self.visited.contains(&state.node) {
                    self.visited.insert(state.node.clone());
                    return Some(state.node.clone());
                }

                while let Some(to) = state.neighbors.next() {
                    if !self.visited.contains(&to) {
                        break 'outer to;
                    }
                }
            }

            self.queue.pop_front();
        };

        self.visited.insert(next.clone());
        self.queue
            .push_back(NodeState::new(self.graph, next.clone()));
        Some(next)
    }
}
