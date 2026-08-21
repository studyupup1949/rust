use abstractgraph;
use abstractgraph::DirectedGraph;
use abstractgraph::graphs::SimpleOutboundEdge;
use abstractgraph::graphs::SimpleOutboundEdges;

// Grid graph(s)
//
// A -> B -> C
// |    |    |
// v    v    v
// D -> E -> F
// |    |    |
// v    v    v
// G -> H -> I
//
// nx * ny nodes arranged in an nx * ny grid.  Depending on
// parameters, edges to the node to the right / down / left / up of
// each node
//
#[derive(Debug)]
pub struct Grid {
    nx : u16,
    ny : u16,
    right : bool,
    down : bool,
    left : bool,
    up : bool,
}

impl Grid {
    pub fn new(nx: u16, ny: u16,
               right: bool, down: bool, left: bool, up: bool) -> Grid {
        Grid {
            nx: nx,
            ny: ny,
            right: right,
            down: down,
            left: left,
            up: up,
        }
    }
}

impl DirectedGraph for Grid {
    type Node = (u16, u16);
    type Edge = SimpleOutboundEdge<Self::Node>;
    type Edges = SimpleOutboundEdges<<Vec<Self::Node> as IntoIterator>::IntoIter>;

    fn edges_from(&self, from: Self::Node) -> Self::Edges {
        let (fx, fy) = from;
        let mut v: Vec<Self::Node> = Vec::with_capacity(4);

        if self.right && fx < (self.nx - 1) {
            v.push((fx + 1, fy));
        }
        if self.down && fy < (self.ny - 1) {
            v.push((fx, fy + 1));
        }
        if self.left  && fx > 0 {
            v.push((fx - 1, fy));
        }
        if self.up && fy > 0 {
            v.push((fx, fy - 1));
        }
        SimpleOutboundEdges::new(v)
    }
}
