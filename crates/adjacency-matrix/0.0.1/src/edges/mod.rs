#[derive(Clone, Debug, Default)]
pub struct AdjacencyEdge<M> {
    pub degree: usize,
    pub metadata: M,
}
