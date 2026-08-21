#![allow(dead_code)]

use itertools::Itertools;
use crate::{
    normed::{UltraNormed, Valuation, ValuationRing},
    traits::AdicPrimitive,
};
use super::Polynomial;


#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// Newton polygon for an adic [`Polynomial`]
pub (crate) struct NewtonPolygon<I>
where I: ValuationRing {
    points: Vec<(usize, Valuation<I>)>,
    edges: Vec<NewtonEdge<I>>,
}


impl<I> NewtonPolygon<I>
where I: ValuationRing {

    /// Create `NewtonPolygon` by analyzing [`Polynomial`] coefficient valuation
    pub fn from_polynomial<A>(poly: &Polynomial<A>) -> Self
    where A: AdicPrimitive + UltraNormed<ValuationRing = I> {

        let points = poly.coefficients().enumerate().map(|(idx, c)| (idx, c.valuation())).collect::<Vec<_>>();
        let edges = newton_polygon_edges(points.clone());
        Self {
            points,
            edges,
        }

    }

    /// Return the vertices of the polygon, NOT including vertices on existing edges
    pub fn vertices(&self) -> impl Iterator<Item = (usize, Valuation<I>)> + use<'_, I> {
        self.edges.iter().map(|e| e.start).chain(self.edges.last().cloned().map(|e| e.end))
    }

}


/// Give the points for the convex hull of the `NewtonPolygon`
fn newton_polygon_edges<I>(points: Vec<(usize, Valuation<I>)>) -> Vec<NewtonEdge<I>>
where I: ValuationRing {

    use Valuation::{Finite, PosInf};

    let Some(first) = points.first().copied() else {
        return vec![];
    };

    // First assemble the hull points, then go back and assemble the edges
    let mut hull_points = vec![first];
    for point in &points[1..] {

        let point = *point;
        let mut should_add_point = true;
        while hull_points.len() > 1 {

            let hull_len = hull_points.len();
            let prev_point = hull_points[hull_len-1];
            let prev_prev_point = hull_points[hull_len-2];
            match (prev_prev_point, prev_point, point) {
                ((_, PosInf), (_, PosInf), (_, _)) => {
                    hull_points.remove(hull_len-2);
                },
                ((_, PosInf), (_, Finite(_)), (_, _)) => {
                    break;
                },
                ((_, Finite(_)), (_, PosInf), (_, PosInf)) => {
                    should_add_point = false;
                    break;
                },
                ((_, Finite(_)), (_, PosInf), (_, Finite(_))) => {
                    hull_points.pop();
                },
                ((_, Finite(_)), (_, Finite(_)), (_, PosInf)) => {
                    break;
                },
                ((pp_idx, Finite(pp_height)), (p_idx, Finite(p_height)), (idx, Finite(height))) => {
                    let new_hdiff = height - pp_height;
                    let old_hdiff = p_height - pp_height;
                    let new_idiff = I::try_from_usize(idx - pp_idx).expect("convert I to usize");
                    let old_idiff = I::try_from_usize(p_idx - pp_idx).expect("convert I to usize");
                    if new_hdiff * old_idiff <= old_hdiff * new_idiff {
                        hull_points.pop();
                    } else {
                        break;
                    }
                },
            }

        }

        if should_add_point {
            hull_points.push(point);
        }

    }

    // Add an extra infinite hull point at the end
    if hull_points.last().is_some_and(|hp| hp.1.is_finite()) {
        hull_points.push((points.len(), PosInf));
    }

    // Now go back through and create NewtonEdges
    let mut point_iter = points.into_iter();
    let edges = hull_points.into_iter().tuple_windows().map(|(hp_start, hp_end)| {
        let mut inner = vec![];
        for point in point_iter.by_ref() {

            if point == hp_start {
                continue;
            }
            if point == hp_end {
                break;
            }

            let point_from_edge = match (hp_start, hp_end, point) {
                ((_, PosInf), _, _) | (_, (_, PosInf), _) | (_, _, (_, PosInf)) => PointFromEdge::AboveEdge,
                ((istart, Finite(hstart)), (iend, Finite(hend)), (ipoint, Finite(hpoint))) => {
                    let hp_hdiff = hend - hstart;
                    let point_hdiff = hpoint - hstart;
                    let hp_idiff = I::try_from_usize(iend - istart).expect("convert I to usize");
                    let point_idiff = I::try_from_usize(ipoint - istart).expect("convert I to usize");
                    if hp_hdiff * point_idiff == point_hdiff * hp_idiff {
                        PointFromEdge::OnEdge
                    } else {
                        PointFromEdge::AboveEdge
                    }
                },
            };

            inner.push((point.0, point.1, point_from_edge));

        }
        NewtonEdge{ start: hp_start, end: hp_end, inner }
    });

    edges.collect()

}


#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct NewtonEdge<I>
where I: ValuationRing {
    start: (usize, Valuation<I>),
    end: (usize, Valuation<I>),
    inner: Vec<(usize, Valuation<I>, PointFromEdge)>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum PointFromEdge {
    OnEdge,
    AboveEdge,
}
