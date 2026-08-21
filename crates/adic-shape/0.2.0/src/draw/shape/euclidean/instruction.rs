#![allow(clippy::unnecessary_wraps)]

use std::collections::HashMap;
use itertools::{Either, Itertools};
use petgraph::visit::{Control, DfsEvent, EdgeRef, IntoNodeReferences, depth_first_search};

use crate::{
    draw::element::{AdicEl, PathColor, PathDInstruction, PathEl, PathStroke},
    error::{AdicShapeError, AdicShapeResult},
};
use super::{graph::EuclideanGraph, EuclideanShape};


type Coordinate = (f64, f64);



/// Path instructions (Move and Line) to draw the base tree of the Euclidean
pub (super) fn tree_paths(euclidean_graph: &EuclideanGraph) -> AdicShapeResult<impl Iterator<Item=AdicEl>> {

    // Convert to PathDInstructions
    let mut instructions = HashMap::new();
    let graph = euclidean_graph.petgraph();
    for e in graph.edge_references() {
        let source = graph.node_weight(e.source()).ok_or(AdicShapeError::PetGraph)?;
        let target = graph.node_weight(e.target()).ok_or(AdicShapeError::PetGraph)?;
        let ins = instructions.entry(e.weight().path_group).or_insert(vec![]);
        ins.push(PathDInstruction::Move((source.x, source.y)));
        ins.push(PathDInstruction::Line((target.x, target.y)));
    }

    // Draw the tree with AdicEls
    let sorted_paths = instructions.into_iter().sorted_by_key(|(path_group, _)| *path_group);
    let tree_paths = sorted_paths.into_iter().filter_map(|(path_group, instructions)| {

        if path_group.stroke == PathStroke::NoStroke {
            return None;
        }

        let default_class = "tree-path";
        let color_class = match path_group.color_group {
            PathColor::Default => "tree-path-default".to_string(),
            PathColor::Combined => "tree-path-combined".to_string(),
            PathColor::Color(n) => format!("tree-path-color-{n}"),
        };
        let stroke_class = match path_group.stroke {
            PathStroke::NoStroke => "tree-path-none",
            PathStroke::Solid => "tree-path-solid",
            PathStroke::Dashed => "tree-path-dashed",
        };
        let class = [default_class, &color_class, stroke_class].join(" ");
        Some(AdicEl::Path(PathEl {
            class: Some(class),
            d: instructions,
        }))

    });

    Ok(tree_paths)

}


/// Path instructions (Move and Line) to draw the smallest convex hulls for the euclidean
pub (super) fn scaled_hulls(euclidean_graph: &EuclideanGraph) -> AdicShapeResult<impl Iterator<Item=AdicEl>> {

    if !euclidean_graph.draw_scaled_hulls() {
        return Ok(Either::Left(std::iter::empty()));
    }

    let mut pathds = vec![];
    let graph = euclidean_graph.petgraph();
    for nx in graph.node_references().filter_map(|(nx, nw)| (nw.depth == euclidean_graph.depth() - 1).then_some(nx)) {

        let mut children = graph.neighbors_directed(nx, petgraph::Direction::Outgoing).collect::<Vec<_>>();
        children.sort();
        let coords = children.into_iter().map(
            |cx| (graph[cx].x, graph[cx].y)
        ).collect::<Vec<_>>();

        let mut hull = convex_hull(&coords);

        // Make the hulls slightly bigger to account for fractals beyond: scaling/(scaling-1)
        let scaling = euclidean_graph.scaling_multiplier();
        let Some((min_x, max_x, min_y, max_y)) = hull.iter().map(|pt| (pt.0, pt.0, pt.1, pt.1)).reduce(
            |acc, pt| (
                f64::min(acc.0, pt.0), f64::max(acc.1, pt.1),
                f64::min(acc.2, pt.2), f64::max(acc.3, pt.3),
            )
        ) else {
            continue;
        };
        let (avg_x, avg_y) = (0.5 * (min_x + max_x), 0.5 * (min_y + max_y));
        for pt in &mut hull {
            pt.0 = scaling / (scaling - 1.0) * (pt.0 - avg_x) + avg_x;
            pt.1 = scaling / (scaling - 1.0) * (pt.1 - avg_y) + avg_y;
        }

        let first = *hull.first().expect("Convex hull empty");
        let pathd = std::iter::once(PathDInstruction::Move(first))
            .chain(hull.into_iter().skip(1).map(PathDInstruction::Line))
            .chain(std::iter::once(PathDInstruction::Line(first))).collect::<Vec<_>>();
        pathds.extend(pathd);

    }

    let path_el = AdicEl::Path(PathEl{
        class: Some("euclidean-convex-hull".to_string()),
        d: pathds
    });

    Ok(Either::Right(std::iter::once(path_el)))

}





/// Path instructions (Move and Line) to draw the valuation lines as convex hulls
pub (super) fn valuation_hulls(euclidean_shape: &EuclideanShape) -> AdicShapeResult<impl Iterator<Item=AdicEl>> {

    let mut level_paths = vec![];

    let num_levels = usize::try_from(euclidean_shape.max_valuation() - euclidean_shape.min_valuation())?;

    let zidx = euclidean_shape.zero_valuation_idx();
    for idx in 0..num_levels {
        let class = if euclidean_shape.show_zero_val_hull() && zidx.is_some_and(|i| idx == i) {
            Some("euclidean-zero-val-hull".to_string())
        } else if euclidean_shape.show_val_hulls() {
            Some("euclidean-val-hull".to_string())
        } else {
            None
        };
        if let Some(class) = class {

            let hull_depth = isize::try_from(idx)? + euclidean_shape.min_valuation();
            let graph = euclidean_shape.graph().petgraph();
            let mut hull_points = vec![];
            depth_first_search(graph, Some(euclidean_shape.graph().root_idx()), |event| -> Control<()> {
                match event {
                    DfsEvent::Discover(parent, _) if graph[parent].depth == hull_depth - 1 => {
                        hull_points.clear();
                        Control::Continue
                    },
                    DfsEvent::Discover(child, _) if graph[child].depth == hull_depth => {
                        hull_points.push((graph[child].x, graph[child].y));
                        Control::Prune
                    },
                    DfsEvent::Finish(parent, _) if graph[parent].depth == hull_depth - 1 => {
                        if !hull_points.is_empty() {
                            let hull = convex_hull(&hull_points);
                            let first = *hull.first().expect("Convex hull empty");
                            let pathd = std::iter::once(PathDInstruction::Move(first))
                                .chain(hull.into_iter().skip(1).map(PathDInstruction::Line))
                                .chain(std::iter::once(PathDInstruction::Line(first))).collect::<Vec<_>>();
                            level_paths.push(AdicEl::Path(PathEl {
                                class: Some(class.clone()),
                                d: pathd,
                            }));
                        }
                        Control::Continue
                    },
                    _ => Control::Continue
                }
            });

        }
    }

    Ok(level_paths.into_iter())

}


/// Path instructions (Move and Line) to draw dots at the smallest elements of the euclidean
pub (super) fn scaled_dots(euclidean_graph: &EuclideanGraph) -> AdicShapeResult<impl Iterator<Item=AdicEl>> {

    if !euclidean_graph.draw_scaled_dots() {
        return Ok(Either::Left(std::iter::empty()));
    }

    // Assuming a dot of depth -1 would have a radius of `1.0`, scale it down
    let radius = 1.0;

    let mut svg_els = vec![];
    let graph = euclidean_graph.petgraph();
    for (nx, nw) in graph.node_references().filter(|(_, nw)| nw.depth == euclidean_graph.depth()) {
        svg_els.push(AdicEl::Circle(crate::draw::element::CircleEl {
            class: Some("euclidean-dot".to_string()),
            cx: graph[nx].x,
            cy: graph[nx].y,
            r: radius / nw.scaling,
        }));
    }

    Ok(Either::Right(svg_els.into_iter()))

}


/// Path instructions (Move and Line) to draw enclosing disks around elements at the given depths
pub (super) fn enclosing_disks(euclidean_graph: &EuclideanGraph) -> AdicShapeResult<impl Iterator<Item=AdicEl>> {

    // Assuming a dot of depth -1 would have a radius of `1.0`, scale it down
    let radius = 1.0;

    let mut disks = vec![];
    for &depth in euclidean_graph.enclosing_disks() {

        let graph = euclidean_graph.petgraph();
        for (nx, nw) in graph.node_references().filter(|(_, nw)| nw.depth == depth-1) {
            disks.push(AdicEl::Circle(crate::draw::element::CircleEl {
                class: Some("euclidean-enclosing-disk".to_string()),
                cx: graph[nx].x,
                cy: graph[nx].y,
                r: radius / nw.scaling,
            }));
        }

    }

    Ok(disks.into_iter())

}


fn convex_hull(vec_pts: &[Coordinate]) -> Vec<Coordinate> {

    // Calculate the convex hull with Graham scan, https://en.wikipedia.org/wiki/Graham_scan

    // Find the lowest y-coordinate and leftmost point, called P0
    let Some(lowest) = vec_pts.iter().min_by(|pt0, pt1| pt0.1.total_cmp(&pt1.1)) else {
        panic!("Could not calculate lowest point of convex hull for Euclidean");
    };
    // Sort points by polar angle with P0, if several points have the same polar angle then only keep the farthest
    let lowest_angle_cmp = |pt0: &Coordinate, pt1: &Coordinate| angle_cmp(lowest, pt0, pt1);
    let mut angle_sorted_vec = vec_pts.to_owned();
    angle_sorted_vec.sort_by(&lowest_angle_cmp);

    // for point in points:
    //     # pop the last point from the stack if we turn clockwise to reach this point
    //     while count stack > 1 and ccw(next_to_top(stack), top(stack), point) <= 0:
    //         pop stack
    //     push point to stack
    // end
    let mut hull_stack = vec![];
    for pt in angle_sorted_vec {
        while let len = hull_stack.len() && len >= 2 {
            let ref_pt = hull_stack[len-2];
            let stack_pt = hull_stack[len-1];
            if angle_cmp(&ref_pt, &stack_pt, &pt).is_ge() {
                hull_stack.pop();
            } else {
                break;
            }
        }
        hull_stack.push(pt);
    }

    hull_stack

}

#[allow(clippy::float_cmp)]
fn angle_cmp(reference: &Coordinate, pt0: &Coordinate, pt1: &Coordinate) -> std::cmp::Ordering {
    if pt0.0 == pt1.0 && pt0.1 == pt1.1 {
        // If pts match, they are equal
        std::cmp::Ordering::Equal
    } else if pt0.0 == reference.0 && pt0.1 == reference.1 {
        // If pt0 matches `reference`, pt0 < pt1
        std::cmp::Ordering::Less
    } else if pt1.0 == reference.0 && pt1.1 == reference.1 {
        // If pt1 matches `reference`, pt0 > pt1
        std::cmp::Ordering::Greater
    } else {
        // Otherwise, compare slopes: y0/x0 vs y1/x1
        let x0 = pt0.0 - reference.0;
        let x1 = pt1.0 - reference.0;
        let y0 = pt0.1 - reference.1;
        let y1 = pt1.1 - reference.1;
        (y0 * x1).total_cmp(&(y1 * x0))
    }
}



#[cfg(test)]
mod test {
    use super::convex_hull;

    #[test]
    fn manual_hulls() {

        // Koch
        let vec_pts = vec![(0.0, 0.0), (1.0, 1.0), (0.5, 0.5), (1.0, 0.0), (0.0, 1.0)];
        let hull = convex_hull(&vec_pts);
        assert_eq!(hull, vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);

        // Hexaflake
        let sqrt3_over_6 = 3.0_f64.sqrt() / 6.0;
        let vec_pts = vec![
            (0.5, 0.5),
            (0.5 - sqrt3_over_6, 0.0), (0.5 + sqrt3_over_6, 0.0), (1.0, 0.5),
            (0.5 + sqrt3_over_6, 1.0), (0.5 - sqrt3_over_6, 1.0), (0.0, 0.5),
        ];
        let hull = convex_hull(&vec_pts);
        assert_eq!(hull, vec![
            (0.5 - sqrt3_over_6, 0.0), (0.5 + sqrt3_over_6, 0.0), (1.0, 0.5),
            (0.5 + sqrt3_over_6, 1.0), (0.5 - sqrt3_over_6, 1.0), (0.0, 0.5),
        ]);
        let vec_pts = vec![
            (50.0, 50.0),
            (21.132486540518716, 0.0), (78.86751345948129, 0.0), (100.0, 50.0),
            (78.86751345948129, 100.0), (21.132486540518716, 100.0), (0.0, 50.0),
        ];
        let hull = convex_hull(&vec_pts);
        assert_eq!(hull, vec![
            (21.132486540518716, 0.0), (78.86751345948129, 0.0), (100.0, 50.0),
            (78.86751345948129, 100.0), (21.132486540518716, 100.0), (0.0, 50.0),
        ]);

    }

}
