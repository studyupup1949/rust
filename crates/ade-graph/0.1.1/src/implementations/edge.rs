use ade_traits::EdgeTrait;

#[derive(Debug, Clone)]
pub struct Edge {
    source: u32,
    target: u32,
}

impl Edge {
    pub fn new(source: u32, target: u32) -> Self {
        Edge { source, target }
    }

    /// Returns the key used for HashMap storage
    pub fn key(&self) -> (u32, u32) {
        (self.source, self.target)
    }

    /// Creates a key from source and target IDs
    pub fn make_key(source: u32, target: u32) -> (u32, u32) {
        (source, target)
    }
}

impl EdgeTrait for Edge {
    fn new(source: u32, target: u32) -> Self {
        Edge { source, target }
    }

    fn source(&self) -> u32 {
        self.source
    }

    fn target(&self) -> u32 {
        self.target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let edge = Edge::new(1, 2);
        assert_eq!(edge.source(), 1);
        assert_eq!(edge.target(), 2);
    }

    #[test]
    fn test_source() {
        let edge = Edge::new(5, 10);
        assert_eq!(edge.source(), 5);
    }

    #[test]
    fn test_target() {
        let edge = Edge::new(5, 10);
        assert_eq!(edge.target(), 10);
    }

    #[test]
    fn test_key() {
        let edge = Edge::new(1, 2);
        assert_eq!(edge.key(), (1, 2));
    }

    #[test]
    fn test_make_key() {
        let key = Edge::make_key(3, 4);
        assert_eq!(key, (3, 4));
    }

    #[test]
    fn test_key_consistency() {
        let edge = Edge::new(7, 8);
        let manual_key = Edge::make_key(7, 8);
        assert_eq!(edge.key(), manual_key);
    }

    #[test]
    fn test_different_edges_different_keys() {
        let edge1 = Edge::new(1, 2);
        let edge2 = Edge::new(2, 1);
        let edge3 = Edge::new(1, 3);

        assert_ne!(edge1.key(), edge2.key());
        assert_ne!(edge1.key(), edge3.key());
        assert_ne!(edge2.key(), edge3.key());
    }

    #[test]
    fn test_same_edges_same_keys() {
        let edge1 = Edge::new(1, 2);
        let edge2 = Edge::new(1, 2);

        assert_eq!(edge1.key(), edge2.key());
        assert_eq!(edge1.source(), edge2.source());
        assert_eq!(edge1.target(), edge2.target());
    }

    #[test]
    fn test_edge_trait_implementation() {
        let edge = Edge::new(42, 99);

        // Test that EdgeTrait methods work correctly
        assert_eq!(edge.source(), 42);
        assert_eq!(edge.target(), 99);
    }
}
