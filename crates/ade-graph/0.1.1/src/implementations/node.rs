use ade_traits::NodeTrait;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Node {
    key: u32,
    predecessors: HashSet<u32>,
    successors: HashSet<u32>,
}

impl Node {
    pub fn new(key: u32) -> Self {
        Node {
            key,
            predecessors: HashSet::new(),
            successors: HashSet::new(),
        }
    }
}

impl NodeTrait for Node {
    fn new(key: u32) -> Self {
        Node::new(key)
    }

    fn key(&self) -> u32 {
        self.key
    }

    fn fresh_copy(&self) -> Self {
        Node {
            key: self.key,
            predecessors: HashSet::new(),
            successors: HashSet::new(),
        }
    }

    fn predecessors(&self) -> &HashSet<u32> {
        &self.predecessors
    }

    fn successors(&self) -> &HashSet<u32> {
        &self.successors
    }

    fn add_predecessor(&mut self, key: u32) {
        self.predecessors.insert(key);
    }

    fn add_successor(&mut self, key: u32) {
        self.successors.insert(key);
    }

    fn remove_predecessor(&mut self, key: u32) {
        self.predecessors.remove(&key);
    }

    fn remove_successor(&mut self, key: u32) {
        self.successors.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(1);
        assert_eq!(node.key(), 1);
        assert!(node.predecessors().is_empty());
        assert!(node.successors().is_empty());
    }

    #[test]
    fn test_fresh_copy() {
        let mut node = Node::new(42);
        node.add_predecessor(1);
        node.add_successor(2);

        let fresh_node = node.fresh_copy();

        assert_eq!(fresh_node.key(), 42);
        assert!(fresh_node.predecessors().is_empty());
        assert!(fresh_node.successors().is_empty());
    }

    #[test]
    fn test_add_predecessor() {
        let mut node = Node::new(1);
        node.add_predecessor(2);
        assert!(node.predecessors().contains(&2));
    }

    #[test]
    fn test_add_successor() {
        let mut node = Node::new(1);
        node.add_successor(2);
        assert!(node.successors().contains(&2));
    }

    #[test]
    fn test_remove_predecessor() {
        let mut node = Node::new(1);
        node.add_predecessor(2);
        node.remove_predecessor(2);
        assert!(!node.predecessors().contains(&2));
    }

    #[test]
    fn test_remove_successor() {
        let mut node = Node::new(1);
        node.add_successor(2);
        node.remove_successor(2);
        assert!(!node.successors().contains(&2));
    }

    #[test]
    fn test_multiple_connections() {
        let mut node = Node::new(1);

        // Add multiple predecessors and successors
        node.add_predecessor(2);
        node.add_predecessor(3);
        node.add_successor(4);
        node.add_successor(5);

        assert_eq!(node.predecessors().len(), 2);
        assert_eq!(node.successors().len(), 2);
        assert!(node.predecessors().contains(&2));
        assert!(node.predecessors().contains(&3));
        assert!(node.successors().contains(&4));
        assert!(node.successors().contains(&5));

        // Remove some connections
        node.remove_predecessor(2);
        node.remove_successor(4);

        assert_eq!(node.predecessors().len(), 1);
        assert_eq!(node.successors().len(), 1);
        assert!(!node.predecessors().contains(&2));
        assert!(node.predecessors().contains(&3));
        assert!(!node.successors().contains(&4));
        assert!(node.successors().contains(&5));
    }

    #[test]
    fn test_duplicate_connections() {
        let mut node = Node::new(1);

        // Adding the same predecessor twice should not duplicate
        node.add_predecessor(2);
        node.add_predecessor(2);
        assert_eq!(node.predecessors().len(), 1);

        // Adding the same successor twice should not duplicate
        node.add_successor(3);
        node.add_successor(3);
        assert_eq!(node.successors().len(), 1);
    }
}
