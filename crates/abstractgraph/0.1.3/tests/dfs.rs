use std::fmt::Debug;
use std::hash::Hash;

use abstractgraph::DirectedGraph;

fn test_dfs<G>(g: &G, start: <G as DirectedGraph>::Node, expected: &[<G as DirectedGraph>::Node])
where
    G: DirectedGraph,
    <G as DirectedGraph>::Node : Debug+Eq+Hash,
{
    let actual : Vec<_> = g.dfs(start).collect();

    assert_eq!(actual, expected);
}

mod graphs;

#[test]
fn trivial() {
    use graphs::trivial;
    let t = trivial::Trivial();
    test_dfs(&t, (), &[()]);
}

#[test]
fn parallel2() {
    use graphs::parallel;
    let p2 = parallel::Parallel::new(2);
    test_dfs(&p2, parallel::Node::A, &[parallel::Node::A, parallel::Node::B]);
    test_dfs(&p2, parallel::Node::B, &[parallel::Node::B]);
}

#[test]
fn full5() {
    use graphs::full;
    let f5 = full::Full::new(5);

    test_dfs(&f5, 0, &[0, 1, 2, 3, 4]);
    test_dfs(&f5, 1, &[1, 0, 2, 3, 4]);
    test_dfs(&f5, 2, &[2, 0, 1, 3, 4]);
    test_dfs(&f5, 3, &[3, 0, 1, 2, 4]);
    test_dfs(&f5, 4, &[4, 0, 1, 2, 3]);
}

#[test]
fn chain8() {
    use graphs::chain;
    let c8 = chain::Chain::new(8);

    test_dfs(&c8, 0, &[0, 1, 2, 3, 4, 5, 6, 7]);
    test_dfs(&c8, 7, &[7, 6, 5, 4, 3, 2, 1, 0]);
    test_dfs(&c8, 4, &[4, 3, 2, 1, 0, 5, 6, 7]);
}

#[test]
fn grid3_rightdown() {
    use graphs::grid;
    let g3 = grid::Grid::new(3, 3, true, true, false, false);

    test_dfs(&g3, (0, 0), &[(0, 0), (1, 0), (2, 0), (2, 1), (2, 2), (1, 1), (1, 2), (0, 1), (0, 2)]);
    test_dfs(&g3, (1, 1), &[(1, 1), (2, 1), (2, 2), (1, 2)]);
    test_dfs(&g3, (2, 2), &[(2, 2)]);
}

#[test]
fn grid3_full() {
    use graphs::grid;
    let g3 = grid::Grid::new(3, 3, true, true, true, true);

    test_dfs(&g3, (0, 0), &[(0, 0), (1, 0), (2, 0), (2, 1), (2, 2), (1, 2), (0, 2), (0, 1), (1, 1)]);
    test_dfs(&g3, (1, 1), &[(1, 1), (2, 1), (2, 2), (1, 2), (0, 2), (0, 1), (0, 0), (1, 0), (2, 0)]);
}
