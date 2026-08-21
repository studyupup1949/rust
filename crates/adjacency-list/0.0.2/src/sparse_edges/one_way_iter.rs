use crate::utils::ShortEdge;
use graph_types::IndeterminateEdge;
use std::collections::{btree_map, btree_set, btree_set::Iter, BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct EdgeFirstAllNodes<'i> {
    pub(crate) nodes: btree_set::Iter<'i, u32>,
}
#[derive(Debug)]
pub struct EdgeFirstAllEdges<'i> {
    pub(crate) edges: btree_map::Iter<'i, u32, ShortEdge>,
}

#[derive(Debug)]
pub struct EdgeFirstAllBridges<'i> {
    pub(crate) edges: btree_map::Iter<'i, u32, ShortEdge>,
}

#[derive(Debug)]
pub struct EdgeFirstFindBridges<'i> {
    pub(crate) edges: btree_map::Iter<'i, u32, ShortEdge>,
    pub(crate) target: ShortEdge,
}

#[derive(Debug)]
pub struct EdgeFirstFindNeighbors<'i> {
    pub(crate) edges: btree_map::Iter<'i, u32, ShortEdge>,
    pub(crate) target: u32,
}

impl<'i> Iterator for EdgeFirstAllNodes<'i> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.nodes.next().map(|x| *x as usize)
    }
}

impl<'i> DoubleEndedIterator for EdgeFirstAllNodes<'i> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.nodes.next_back().map(|x| *x as usize)
    }
}

impl<'i> Iterator for EdgeFirstAllEdges<'i> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.edges.next().map(|(x, _)| *x as usize)
    }
}

impl<'i> DoubleEndedIterator for EdgeFirstAllEdges<'i> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.edges.next_back().map(|(x, _)| *x as usize)
    }
}

impl<'i> Iterator for EdgeFirstAllBridges<'i> {
    type Item = IndeterminateEdge;

    fn next(&mut self) -> Option<Self::Item> {
        self.edges.next().map(|(_, y)| y.as_indeterminate())
    }
}

impl<'i> DoubleEndedIterator for EdgeFirstAllBridges<'i> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.edges.next_back().map(|(_, y)| y.as_indeterminate())
    }
}

impl<'i> Iterator for EdgeFirstFindBridges<'i> {
    type Item = IndeterminateEdge;

    fn next(&mut self) -> Option<Self::Item> {
        let (id, edge) = self.edges.next()?;
        if self.target.eq(edge) { Some(edge.as_indeterminate()) } else { self.next() }
    }
}

impl<'i> DoubleEndedIterator for EdgeFirstFindBridges<'i> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (id, edge) = self.edges.next_back()?;
        if self.target.eq(edge) { Some(edge.as_indeterminate()) } else { self.next_back() }
    }
}

impl<'i> Iterator for EdgeFirstFindNeighbors<'i> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'i> DoubleEndedIterator for EdgeFirstFindNeighbors<'i> {
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
