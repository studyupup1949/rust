use crate::{DiscoveredSet, NodeCost};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// A binary min-heap implementation of the discovered set.
#[derive(Clone, Debug, Default)]
pub struct MinHeap<Node, Cost>
where
    Node: Copy,
    Cost: Copy + Ord,
{
    heap: BinaryHeap<Reverse<NodeCost<Node, Cost>>>,
}

impl<Node, Cost> DiscoveredSet<Node, Cost> for MinHeap<Node, Cost>
where
    Node: Copy,
    Cost: Copy + Ord,
{
    fn push(&mut self, node: NodeCost<Node, Cost>) {
        self.heap.push(Reverse(node));
    }

    fn pop(&mut self) -> Option<NodeCost<Node, Cost>> {
        self.heap.pop().map(|Reverse(node)| node)
    }

    fn clear(&mut self) {
        self.heap.clear()
    }
}
