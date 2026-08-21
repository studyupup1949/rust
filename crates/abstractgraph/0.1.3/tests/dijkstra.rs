use num;

use abstractgraph::weight::WeightedDirectedGraph;
use abstractgraph::weight::UnitWeightGraph;
use abstractgraph::weight::WeightedOutboundEdge;
use abstractgraph::algorithms::dijkstra;

fn test_path<G, W>(g: &G, state: &mut dijkstra::Dijkstra<G, W>,
                   start: G::Node, end: G::Node)
where
    G: WeightedDirectedGraph<W>+std::fmt::Debug,
    G::Node: Eq+std::hash::Hash+std::fmt::Debug,
    G::Edge: Eq+std::hash::Hash+Clone+std::fmt::Debug,
    G::Edge: abstractgraph::weight::WeightedOutboundEdge<G::Node, W>,
    W: Ord+num::Zero+Clone+std::fmt::Debug,
    W: for<'b> std::ops::Add<&'b W, Output=W>,
{
    let distance = state.distance(end.clone()).unwrap();
    let prev = state.prev(end.clone()).unwrap();

    if end == start {
        assert!(distance.is_zero());
        assert_eq!(prev, None);
        return;
    }

    let (pnode, pedge) = prev.unwrap();

    for e in g.edges_from(pnode.clone()) {
        let prevdistance = state.distance(pnode.clone()).unwrap();

        if e == pedge {
            assert_eq!(prevdistance + pedge.weight(), distance);
            return;
        }
    }

    // Edge that Dijkstra found wasn't actually there, oops
    unreachable!();
}

fn test_dijkstra<G, W>(g: &G, start: G::Node)
where
    G: WeightedDirectedGraph<W>+std::fmt::Debug,
    G::Node: Eq+std::hash::Hash+std::fmt::Debug,
    G::Edge: Eq+std::hash::Hash+Clone+std::fmt::Debug,
    G::Edge: abstractgraph::weight::WeightedOutboundEdge<G::Node, W>,
    W: Ord+num::Zero+Clone+std::fmt::Debug,
    W: for<'b> std::ops::Add<&'b W, Output=W>,
{
    let mut d = dijkstra::Dijkstra::new(g);
    let mut last_distance: W;

    d.add_source(start.clone());

    assert_eq!(d.next(), Some(start.clone()));
    last_distance = d.distance(start.clone()).unwrap();
    assert_eq!(last_distance, W::zero());

    while let Some(n) = d.next() {
        let next_distance = d.distance(n.clone()).unwrap();
        assert!(next_distance >= last_distance);
        test_path(g, &mut d, start.clone(), n);
    }
}

mod graphs;

#[test]
fn trivial() {
    use graphs::trivial;
    let t = UnitWeightGraph::<_, i32>::new(trivial::Trivial());

    test_dijkstra::<_, i32>(&t, ());
}

#[test]
fn parallel2() {
    use graphs::parallel;
    let p2 = UnitWeightGraph::<_, i32>::new(parallel::Parallel::new(2));
    test_dijkstra::<_, i32>(&p2, parallel::Node::A);
    test_dijkstra::<_, i32>(&p2, parallel::Node::B);
}

#[test]
fn full5() {
    use graphs::full;
    let f5 = UnitWeightGraph::<_, i32>::new(full::Full::new(5));

    test_dijkstra::<_, i32>(&f5, 0);
    test_dijkstra::<_, i32>(&f5, 1);
    test_dijkstra::<_, i32>(&f5, 2);
    test_dijkstra::<_, i32>(&f5, 3);
    test_dijkstra::<_, i32>(&f5, 4);
}

#[test]
fn chain8() {
    use graphs::chain;
    let c8 = UnitWeightGraph::<_, i32>::new(chain::Chain::new(8));

    test_dijkstra::<_, i32>(&c8, 0);
    test_dijkstra::<_, i32>(&c8, 7);
    test_dijkstra::<_, i32>(&c8, 4);
}

#[test]
fn grid3_rightdown() {
    use graphs::grid;
    let g3 = UnitWeightGraph::<_, i32>::new(grid::Grid::new(3, 3, true, true, false, false));

    test_dijkstra::<_, i32>(&g3, (0, 0));
    test_dijkstra::<_, i32>(&g3, (1, 1));
    test_dijkstra::<_, i32>(&g3, (2, 2));
}

#[test]
fn grid3_full() {
    use graphs::grid;
    let g3 = UnitWeightGraph::<_, u32>::new(grid::Grid::new(3, 3, true, true, true, true));

    test_dijkstra::<_, i32>(&g3, (0, 0));
    test_dijkstra::<_, i32>(&g3, (1, 1));
}
