use crate::AdjacencyEdge;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct StaticDirected<V = (), E = ()> {
    /// metadata of vertexes
    vertexes: Vec<V>,
    /// edges, lower triangular matrix
    edges: Vec<AdjacencyEdge<E>>,
}

impl<V, E> Display for StaticDirected<V, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let size = self.min_degree().to_string().len();
        let max = self.vertexes.len();
        for i in 0..max {
            for j in 0..max {
                let index = i * max + j;
                let edge = self.edges.get(index).unwrap();
                write!(f, "{:width$} ", edge.degree, width = size)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<V, E> StaticDirected<V, E> {
    pub fn max_degree(&self) -> usize {
        self.edges.iter().map(|edge| edge.degree).max().unwrap_or(0)
    }
    pub fn min_degree(&self) -> usize {
        self.edges.iter().map(|edge| edge.degree).min().unwrap_or(0)
    }
}
