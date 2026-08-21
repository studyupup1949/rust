use std::collections::HashMap;
use itertools::Itertools;
use petgraph::visit::EdgeRef;

use crate::{
    draw::element::{AdicEl, PathColor, PathDInstruction, PathEl, PathStroke},
    error::{AdicShapeError, AdicShapeResult},
    shape::{Direction, DisplayShape},
};
use super::{create, TreeShape};


/// Path instructions (Move and Line) to draw the base tree
///
/// # Errors
/// Errors if graph gets into a bad state
pub (super) fn tree_paths(tree_shape: &TreeShape) -> AdicShapeResult<impl Iterator<Item=AdicEl>> {

    // Convert to PathDInstructions
    let mut instructions = HashMap::new();
    for e in tree_shape.tree_graph().edge_references() {
        let source = tree_shape.tree_graph().node_weight(e.source()).ok_or(AdicShapeError::PetGraph)?;
        let target = tree_shape.tree_graph().node_weight(e.target()).ok_or(AdicShapeError::PetGraph)?;
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


/// Path instructions (Move and Line) to draw the valuation lines perpendicular to the tree
pub (super) fn valuation_levels(tree_shape: &TreeShape) -> AdicShapeResult<impl Iterator<Item=AdicEl>> {

    let mut level_paths = vec![];

    let num_levels = usize::try_from(tree_shape.max_valuation() - tree_shape.min_valuation())?;
    let (root_length, reg_length) = create::calc_branch_lengths(
        tree_shape.base(), num_levels, tree_shape.dangling_direction().is_some()
    )?;
    let mut level_positions = vec![];
    let mut cur_position = root_length;
    for _ in 0..=num_levels {
        level_positions.push(cur_position);
        cur_position += reg_length;
    }

    let (w, h) = (f64::from(tree_shape.viewbox_width()), f64::from(tree_shape.viewbox_height()));
    let level_ends = match tree_shape.direction() {
        Direction::Up => level_positions.into_iter().map(
            |pos| [(0.0, h * (1.0 - pos)), (w, h * (1.0 - pos))],
        ).collect::<Vec<_>>(),
        Direction::Down => level_positions.into_iter().map(
            |pos| [(0.0, h * pos), (w, h * pos)],
        ).collect::<Vec<_>>(),
        Direction::Left => level_positions.into_iter().map(
            |pos| [(w * (1.0 - pos), 0.0), (w * (1.0 - pos), h)],
        ).collect::<Vec<_>>(),
        Direction::Right => level_positions.into_iter().map(
            |pos| [(w * pos, 0.0), (w * pos, h)],
        ).collect::<Vec<_>>(),
    };

    let zidx = tree_shape.zero_valuation_idx();
    for (idx, level_end) in level_ends.into_iter().enumerate() {
        let class = if tree_shape.show_zero_val_level() && zidx.is_some_and(|i| idx == i) {
            Some("tree-zero-val-level".to_string())
        } else if tree_shape.show_val_levels() {
            Some("tree-val-level".to_string())
        } else {
            None
        };
        if let Some(class) = class {
            level_paths.push(AdicEl::Path(PathEl {
                class: Some(class),
                d: vec![PathDInstruction::Move(level_end[0]), PathDInstruction::Line(level_end[1])],
            }));
        }
    }

    Ok(level_paths.into_iter())

}
