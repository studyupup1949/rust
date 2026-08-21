use crate::{EdgeTrait, NodeTrait};

pub trait GraphViewTrait<N: NodeTrait, E: EdgeTrait> {
    fn is_empty(&self) -> bool;
    fn get_node(&self, key: u32) -> &N;
    fn get_edge(&self, source: u32, target: u32) -> &E;
    fn has_node(&self, key: u32) -> bool;
    fn has_edge(&self, source: u32, target: u32) -> bool;
    fn get_nodes<'a>(&'a self) -> impl Iterator<Item = &'a N>
    where
        N: 'a;
    fn get_edges<'a>(&'a self) -> impl Iterator<Item = &'a E>
    where
        E: 'a;
    fn get_predecessors<'a>(&'a self, node_key: u32) -> impl Iterator<Item = &'a N>
    where
        N: 'a;
    fn get_successors<'a>(&'a self, node_key: u32) -> impl Iterator<Item = &'a N>
    where
        N: 'a;
    fn get_node_keys(&self) -> impl Iterator<Item = u32> + '_;
    fn get_predecessors_keys(&self, node_key: u32) -> impl Iterator<Item = u32> + '_;
    fn get_successors_keys(&self, node_key: u32) -> impl Iterator<Item = u32> + '_;
    fn filter(&self, node_keys: &[u32]) -> impl GraphViewTrait<N, E>;
    fn has_sequential_keys(&self) -> bool;
}
