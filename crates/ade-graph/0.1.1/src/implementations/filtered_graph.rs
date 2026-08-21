use crate::implementations::Graph;
use ade_traits::{EdgeTrait, GraphViewTrait, NodeTrait};
use fixedbitset::FixedBitSet;

pub struct FilteredGraph<'a, N: NodeTrait, E: EdgeTrait> {
    base: &'a Graph<N, E>,
    active: FixedBitSet,
}

impl<'a, N: NodeTrait, E: EdgeTrait> FilteredGraph<'a, N, E> {
    pub fn new(base: &'a Graph<N, E>, active_nodes: impl IntoIterator<Item = u32>) -> Self {
        let node_count = base.get_node_keys().count();

        // Assume normalized keys: 0, 1, 2, ..., n-1
        let mut active = FixedBitSet::with_capacity(node_count);
        for key in active_nodes {
            if (key as usize) < node_count {
                active.insert(key as usize);
            }
        }

        Self { base, active }
    }

    fn is_active(&self, key: u32) -> bool {
        self.active.contains(key as usize)
    }
}

impl<N: NodeTrait, E: EdgeTrait> GraphViewTrait<N, E> for FilteredGraph<'_, N, E> {
    fn is_empty(&self) -> bool {
        self.active.count_ones(..) == 0
    }

    fn get_node(&self, key: u32) -> &N {
        if !self.is_active(key) {
            panic!("Node {} not active in filtered graph", key);
        }
        self.base.get_node(key)
    }

    fn has_node(&self, key: u32) -> bool {
        self.is_active(key)
    }

    fn get_nodes<'b>(&'b self) -> impl Iterator<Item = &'b N>
    where
        N: 'b,
    {
        self.base
            .get_nodes()
            .filter(move |n| self.is_active(n.key()))
    }

    fn get_node_keys(&self) -> impl Iterator<Item = u32> {
        self.base
            .get_node_keys()
            .filter(move |&k| self.is_active(k))
    }

    fn get_edge(&self, source: u32, target: u32) -> &E {
        if !self.is_active(source) {
            panic!("Source node {} not active in filtered graph", source);
        }
        if !self.is_active(target) {
            panic!("Target node {} not active in filtered graph", target);
        }
        self.base.get_edge(source, target)
    }

    fn has_edge(&self, source: u32, target: u32) -> bool {
        self.is_active(source) && self.is_active(target) && self.base.has_edge(source, target)
    }

    fn get_edges<'b>(&'b self) -> impl Iterator<Item = &'b E>
    where
        E: 'b,
    {
        self.base
            .get_edges()
            .filter(move |e| self.is_active(e.source()) && self.is_active(e.target()))
    }

    fn get_predecessors<'b>(&'b self, node_key: u32) -> impl Iterator<Item = &'b N>
    where
        N: 'b,
    {
        if !self.is_active(node_key) {
            panic!("Node {} not active in filtered graph", node_key);
        }
        self.base
            .get_node(node_key)
            .predecessors()
            .iter()
            .filter(move |&&pred| self.is_active(pred))
            .map(move |&pred| self.base.get_node(pred))
    }

    fn get_successors<'b>(&'b self, node_key: u32) -> impl Iterator<Item = &'b N>
    where
        N: 'b,
    {
        if !self.is_active(node_key) {
            panic!("Node {} not active in filtered graph", node_key);
        }
        self.base
            .get_node(node_key)
            .successors()
            .iter()
            .filter(move |&&succ| self.is_active(succ))
            .map(move |&succ| self.base.get_node(succ))
    }

    fn get_successors_keys(&self, node_key: u32) -> impl Iterator<Item = u32> {
        if !self.is_active(node_key) {
            panic!("Node {} not active in filtered graph", node_key);
        }
        self.base
            .get_successors_keys(node_key)
            .filter(move |&succ| self.is_active(succ))
    }

    fn get_predecessors_keys(&self, node_key: u32) -> impl Iterator<Item = u32> {
        if !self.is_active(node_key) {
            panic!("Node {} not active in filtered graph", node_key);
        }
        self.base
            .get_predecessors_keys(node_key)
            .filter(move |&pred| self.is_active(pred))
    }

    fn filter(&self, node_keys: &[u32]) -> impl GraphViewTrait<N, E> {
        // Intersect the requested nodes with the currently active ones
        let filtered_keys = node_keys.iter().copied().filter(|&key| self.is_active(key));

        FilteredGraph::new(self.base, filtered_keys)
    }

    fn has_sequential_keys(&self) -> bool {
        let size = self.active.count_ones(..);
        if size == 0 {
            return true;
        }

        // Quick checks first - check if we have nodes 0 and size-1
        if !self.is_active(0) || !self.is_active(size as u32 - 1) {
            return false;
        }

        // Check if all keys from 0 to size-1 are active
        (0..size as u32).all(|i| self.is_active(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementations::{Edge, Node};
    use ade_traits::GraphViewTrait;

    #[test]
    fn test_filtered_graph_has_sequential_keys() {
        let mut base_graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        // Add nodes 0, 1, 2, 3, 4
        for i in 0..5 {
            base_graph.add_node(Node::new(i));
        }

        // Test 1: Filter to nodes 0, 1, 2 (sequential from 0)
        let filtered = FilteredGraph::new(&base_graph, vec![0, 1, 2]);
        assert!(filtered.has_sequential_keys());

        // Test 2: Filter to nodes 1, 2, 3 (not starting from 0)
        let filtered = FilteredGraph::new(&base_graph, vec![1, 2, 3]);
        assert!(!filtered.has_sequential_keys());

        // Test 3: Filter to nodes 0, 2 (not consecutive)
        let filtered = FilteredGraph::new(&base_graph, vec![0, 2]);
        assert!(!filtered.has_sequential_keys());

        // Test 4: Empty filter
        let filtered = FilteredGraph::new(&base_graph, vec![]);
        assert!(filtered.has_sequential_keys());

        // Test 5: Single node 0
        let filtered = FilteredGraph::new(&base_graph, vec![0]);
        assert!(filtered.has_sequential_keys());

        // Test 6: Single node not 0
        let filtered = FilteredGraph::new(&base_graph, vec![2]);
        assert!(!filtered.has_sequential_keys());
    }
}
