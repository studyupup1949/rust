use std::fmt::Debug;

extern crate abstractgraph;
use abstractgraph::DirectedGraph;

struct AdjacencyTable<'a, G: DirectedGraph> (
    &'a [(<G as DirectedGraph>::Node,
          &'a [<G as DirectedGraph>::Node])],
) where
    <G as DirectedGraph>::Node : 'a;

fn test_adjacency<G>(g: &G, at: &AdjacencyTable<G>)
where
    G: DirectedGraph,
    <G as DirectedGraph>::Node : Debug+Eq,
{
    let AdjacencyTable(l) = *at;
    for (from, tolist) in l {
        let neighbors : Vec<_> = g.neighbors(from.clone()).collect();
        assert_eq!(*tolist, &neighbors[..]);
    }
}

mod samplegraphs;

#[test]
fn trivial() {
    use crate::samplegraphs::trivial;
    let t = trivial::Trivial();
    let at = AdjacencyTable::<trivial::Trivial> ( &[
        ((), &[]),
    ]);
    test_adjacency(&t, &at);
}

#[test]
fn parallel2() {
    use crate::samplegraphs::parallel;
    let p2 = parallel::Parallel::new(2);
    let at = AdjacencyTable::<parallel::Parallel> ( &[
        (parallel::Node::A, &[parallel::Node::B, parallel::Node::B]),
        (parallel::Node::B, &[]),
    ]);
    test_adjacency(&p2, &at);
}


#[test]
fn full5() {
    use crate::samplegraphs::full;
    let f5 = full::Full::new(5);
    let at = AdjacencyTable::<full::Full> ( &[
        (0, &[0, 1, 2, 3, 4]),
        (1, &[0, 1, 2, 3, 4]),
        (2, &[0, 1, 2, 3, 4]),
        (3, &[0, 1, 2, 3, 4]),
        (4, &[0, 1, 2, 3, 4]),
    ]);
    test_adjacency(&f5, &at);
}

#[test]
fn chain8() {
    use crate::samplegraphs::chain;
    let c8 = chain::Chain::new(8);
    let at = AdjacencyTable::<chain::Chain> ( &[
        (0, &[1]),
        (1, &[0, 2]),
        (2, &[1, 3]),
        (3, &[2, 4]),
        (4, &[3, 5]),
        (5, &[4, 6]),
        (6, &[5, 7]),
        (7, &[6]),
    ]);
    test_adjacency(&c8, &at);
}

#[test]
fn grid3_rightdown() {
    use crate::samplegraphs::grid;
    let g3 = grid::Grid::new(3, 3, true, true, false, false);
    let at = AdjacencyTable::<grid::Grid> ( &[
        ((0, 0), &[(1, 0), (0, 1)]),
        ((1, 0), &[(2, 0), (1, 1)]),
        ((2, 0), &[(2, 1)]),
        ((0, 1), &[(1, 1), (0, 2)]),
        ((1, 1), &[(2, 1), (1, 2)]),
        ((2, 1), &[(2, 2)]),
        ((0, 2), &[(1, 2)]),
        ((1, 2), &[(2, 2)]),
        ((2, 2), &[]),
    ]);
    test_adjacency(&g3, &at);
}

#[test]
fn grid3_full() {
    use crate::samplegraphs::grid;
    let g3 = grid::Grid::new(3, 3, true, true, true, true);
    let at = AdjacencyTable::<grid::Grid> ( &[
        ((0, 0), &[(1, 0), (0, 1)]),
        ((1, 0), &[(2, 0), (1, 1), (0, 0)]),
        ((2, 0), &[(2, 1), (1, 0)]),
        ((0, 1), &[(1, 1), (0, 2), (0, 0)]),
        ((1, 1), &[(2, 1), (1, 2), (0, 1), (1, 0)]),
        ((2, 1), &[(2, 2), (1, 1), (2, 0)]),
        ((0, 2), &[(1, 2), (0, 1)]),
        ((1, 2), &[(2, 2), (0, 2), (1, 1)]),
        ((2, 2), &[(1, 2), (2, 1)]),
    ]);
    test_adjacency(&g3, &at);
}
