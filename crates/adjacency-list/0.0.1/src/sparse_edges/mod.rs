use graph_types::{
    DirectedEdge, DynamicEdge, Edge, EdgeDirection, EdgeInsertID, EdgeQuery, GetEdgesVisitor, GraphEngine, NodesVisitor,
};
use std::collections::BTreeMap;

type EdgeID = u32;
type StartNodeID = u32;
type EndNodeID = u32;

#[doc = include_str!("AdjacencyEdgeList.html")]
#[derive(Debug)]
pub struct AdjacencyEdgeList {
    edges: BTreeMap<EdgeID, ShortEdge>,
}

#[derive(Debug)]
pub struct ShortEdge {
    from: StartNodeID,
    goto: EndNodeID,
}

impl GraphEngine for AdjacencyEdgeList {
    fn has_node(&self, node_id: usize) -> bool {
        let id = node_id as u32;
        for edge in self.edges.values() {
            if edge.from == id || edge.goto == id {
                return true;
            }
        }
        false
    }

    fn remove_node_with_edges(&mut self, node_id: usize) {
        let id = node_id as u32;

        todo!()
    }

    fn traverse_nodes(&self) -> NodesVisitor<Self> {
        todo!()
    }

    fn get_edges(&self) -> GetEdgesVisitor<Self> {
        GetEdgesVisitor::new(self)
    }

    fn insert_edge_with_nodes<E: Edge>(&mut self, edge: E) -> EdgeInsertID {
        let lhs = edge.lhs() as u32;
        let rhs = edge.rhs() as u32;
        match edge.direction() {
            EdgeDirection::Disconnect => EdgeInsertID::Nothing,
            EdgeDirection::TwoWay => {
                let e1 = self.insert_one_way_edge(lhs, rhs);
                let e2 = self.insert_one_way_edge(rhs, lhs);
                EdgeInsertID::TwoEdges(e1, e2)
            }
            EdgeDirection::Forward => {
                let e1 = self.insert_one_way_edge(lhs, rhs);
                EdgeInsertID::OneEdge(e1)
            }
            EdgeDirection::Reverse => {
                let e1 = self.insert_one_way_edge(rhs, lhs);
                EdgeInsertID::OneEdge(e1)
            }
        }
    }

    fn remove_edge<E>(&mut self, edge: E)
    where
        E: Into<EdgeQuery>,
    {
        match edge.into() {
            EdgeQuery::EdgeID(i) => {
                self.edges.remove(&(i as u32));
            }
            EdgeQuery::Directed(di) => {
                todo!()
            }
            EdgeQuery::Undirected(_) => {
                todo!()
            }
        }
    }

    fn count_edges(&self) -> usize {
        self.edges.len()
    }
}

impl AdjacencyEdgeList {
    pub(crate) fn insert_one_way_edge(&mut self, start: u32, end: u32) -> usize {
        let id = self.edges.len() as u32 + 1;
        self.edges.insert(id, ShortEdge { from: start, goto: end });
        id as usize
    }
}
