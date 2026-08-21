use std::collections::HashMap;

use adic::traits::HasDigits;
use itertools::Itertools;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::{Dfs, EdgeRef},
    Graph,
};

use crate::{
    draw::element::{PathGroup, PathStroke},
    error::{AdicShapeError, AdicShapeResult},
    shape::Direction,
};
use super::{
    graph::{TreeEdge, TreeGraph, TreeNode},
};


use Direction::{Left, Right, Up, Down};


/// Create a "full tree graph", plotting the ENTIRE tree of possible adic numbers
///
/// # Errors
/// Errors for (1) integer conversion failure, (2) inconsistent directions, and (3) petgraph inconsistency
pub (super) fn create_tree_graph(
    base: u32,
    depth: usize,
    direction: Direction,
    dangling_direction: Option<Direction>,
    draw_full_tree: PathStroke,
) -> AdicShapeResult<(TreeGraph, NodeIndex, Option<NodeIndex>)> {

    if draw_full_tree == PathStroke::NoStroke {
        return Err(AdicShapeError::ImproperConfig(
            "Set solid_full_tree() or dashed_full_tree() when plotting the full tree".to_string()
        ))
    }

    let mut graph = Graph::new();
    let default_path_group = PathGroup { stroke: draw_full_tree, ..Default::default() };
    let (root_length, reg_length) = calc_branch_lengths(base, depth, dangling_direction.is_some())?;
    let (root_idx, dangling_idx) = root_tree(
        &mut graph, direction, dangling_direction,
        default_path_group,
        reg_length, root_length,
    )?;

    let mut cur_branch_point = root_idx;
    let mut cur_branch_choice = 0;
    let mut cur_width = 1.;
    let mut cur_node = graph.node_weight(cur_branch_point).ok_or(AdicShapeError::PetGraph)?;
    let mut cur_edge_set: Vec<EdgeIndex> = vec![];
    'tree_pos_loop: loop {

        // If current depth is less than max, go deeper
        if cur_node.depth < depth.try_into()? {

            let adjust_choice = ((f64::from(cur_branch_choice) + 0.5) / f64::from(base) - 0.5) * cur_width;
            let adjust_length = reg_length;
            let adjust_width = cur_width / f64::from(base);

            let (new_x, new_y) = match direction {
                Up => (cur_node.x + adjust_choice, cur_node.y - adjust_length),
                Down => (cur_node.x - adjust_choice, cur_node.y + adjust_length),
                Left => (cur_node.x - adjust_length, cur_node.y - adjust_choice),
                Right => (cur_node.x + adjust_length, cur_node.y + adjust_choice),
            };

            let (new_idx, edge_idx) = add_branch_from(
                &mut graph, cur_branch_point, cur_branch_choice,
                new_x, new_y, 1.0 / f64::from(base), 1.0,
                default_path_group,
            )?;

            cur_branch_point = new_idx;
            cur_branch_choice = 0;
            cur_width = adjust_width;
            cur_node = graph.node_weight(new_idx).ok_or(AdicShapeError::PetGraph)?;
            cur_edge_set.push(edge_idx);

        }
        // Else, pop up until the next branch choice is available or we're done with the graph
        else {

            while (cur_node.depth >= depth.try_into()?) || (cur_branch_choice == base - 1) {

                // If we're at the root, we're done
                if cur_node.depth == 0 {
                    break 'tree_pos_loop;
                }

                let old_edge_idx = cur_edge_set.pop().ok_or(AdicShapeError::PetGraph)?;
                cur_branch_point = graph.edge_endpoints(old_edge_idx).ok_or(AdicShapeError::PetGraph)?.0;
                cur_branch_choice = graph.edge_weight(old_edge_idx).ok_or(AdicShapeError::PetGraph)?.branch_choice;
                cur_width = cur_width * f64::from(base);

                cur_node = graph.node_weight(cur_branch_point).ok_or(AdicShapeError::PetGraph)?;

            }

            cur_branch_choice += 1;

        }

    }

    Ok((graph, root_idx, dangling_idx))

}


/// Create a "zoomed tree graph", following the digits of `adic_data` without plotting the ENTIRE tree
///
/// # Errors
/// Errors for (1) integer conversion failure, (2) inconsistent directions, and (3) petgraph inconsistency
pub (super) fn create_split_tree_graph(
    adic_data: Vec<&impl HasDigits>,
    depth: usize,
    direction: Direction,
    dangling_direction: Option<Direction>,
    twig_depth: u32,
) -> AdicShapeResult<(TreeGraph, NodeIndex, Option<NodeIndex>)> {

    struct JointBranchAdic<I>
    where I: Iterator<Item = u32> {
        adic_iters: Vec<I>,
        current_branch_point: NodeIndex,
    }

    // Precheck and calculate base
    let Some(first) = adic_data.first() else {
        Err(AdicShapeError::InsufficientData("Cannot create split tree graph with no data".to_string()))?
    };
    let base = u32::from(first.base());

    // Set up (possibly rooted) graph
    let mut graph = Graph::new();
    let default_path_group = PathGroup::default();
    let (root_length, reg_length) = calc_branch_lengths(base, depth, dangling_direction.is_some())?;
    let (root_idx, dangling_idx) = root_tree(
        &mut graph, direction, dangling_direction,
        default_path_group,
        reg_length, root_length,
    )?;

    // Set up structure to store intermediate branch splitting and position
    let mut joint_branches = vec![JointBranchAdic {
        adic_iters: adic_data.into_iter().map(
            |a| a.digits().take(depth).collect::<Vec<_>>().into_iter()
        ).collect(),
        current_branch_point: root_idx,
    }];

    // Build the tree up to depth
    for current_depth in 0..depth {

        // First, calculate branch choices per CURRENT branch and total NEW branch iterators of digits

        let each_branch_point_and_branches = joint_branches.into_iter().map(|joint_branch| {
            let mut new_branch_hash = HashMap::<u32, Vec<_>>::new();
            for mut a in joint_branch.adic_iters {
                let choice = a.next().unwrap_or(0);
                if let Some(jb) = new_branch_hash.get_mut(&choice) {
                    jb.push(a);
                } else {
                    new_branch_hash.insert(choice, vec![a]);
                }
            }
            let new_branches = new_branch_hash.into_iter().sorted_by_key(|(choice, _)| *choice).collect::<Vec<_>>();
            (joint_branch.current_branch_point, new_branches)
        }).collect::<Vec<_>>();

        // Now that we know the new number of branches, we can calculate the positions and add nodes
        let num_old_branches = each_branch_point_and_branches.len();
        let num_new_branches = each_branch_point_and_branches.iter().map(
            |(_, adic_iters)| adic_iters.len()
        ).sum::<usize>();

        // Calculate various lengths
        let width_scale = f64::from(u32::try_from(num_old_branches)?) / f64::from(u32::try_from(num_new_branches)?);
        let length_scale = 1.0;
        let branch_width = 0.5 / f64::from(u32::try_from(num_new_branches)?);
        let tree_length = root_length + f64::from(u32::try_from(current_depth + 1)?) * reg_length;

        // Loop through, add branches, and update joint_branches
        let mut which_new_branch = 0;
        joint_branches = Vec::with_capacity(num_new_branches);
        for (branch_point, branches) in each_branch_point_and_branches {
            for (branch_choice, adic_iters) in branches {

                let fan_center = (f64::from(which_new_branch) + 0.5) / f64::from(u32::try_from(num_new_branches)?);
                let branch_adjustment = ((f64::from(branch_choice) + 0.5) / f64::from(base) - 0.5) * branch_width;
                let endpoint_breadth = fan_center + branch_adjustment;
                let endpoint_length = tree_length;
                let (endpoint_x, endpoint_y) = match direction {
                    Up => (endpoint_breadth, 1.0 - endpoint_length),
                    Down => (1.0 - endpoint_breadth, endpoint_length),
                    Left => (1.0 - endpoint_length, 1.0 - endpoint_breadth),
                    Right => (endpoint_length, endpoint_breadth),
                };

                let (next_idx, _edge_idx) = add_branch_from(
                    &mut graph, branch_point, branch_choice,
                    endpoint_x, endpoint_y, width_scale, length_scale,
                    default_path_group,
                )?;

                which_new_branch += 1;
                joint_branches.push(JointBranchAdic { adic_iters, current_branch_point: next_idx });

            }
        }

    }

    decorate_with_twigs(&mut graph, root_idx, base, direction, reg_length, twig_depth, default_path_group)?;

    Ok((graph, root_idx, dangling_idx))

}


const TWIG_WIDTH_SCALE: f64 = 0.4;
const TWIG_LENGTH_SCALE: f64 = 0.4;

fn decorate_with_twigs(
    graph: &mut TreeGraph, root_idx: NodeIndex, base: u32,
    direction: Direction, branch_length: f64,
    twig_depth: u32,
    path_group: PathGroup,
) -> AdicShapeResult<()> {

    if twig_depth == 0 {
        return Ok(());
    }

    // Decorate with twigs for unchosen branches
    // The top level of twigs have complicated construction; further in twigs are simpler, p-fans
    let mut search = Dfs::new(&*graph, root_idx);
    while let Some(branch_point_idx) = search.next(&*graph) {

        let node_ref = &graph[branch_point_idx];
        let branch_point_x = node_ref.x;
        let branch_point_y = node_ref.y;
        let branch_width = node_ref.branch_width;
        let main_branches = graph.edges(branch_point_idx);
        let choices_and_positions = main_branches.map(|e| {
            let choice = e.weight().branch_choice;
            let endpoint = &graph[e.target()];
            (choice, endpoint.x, endpoint.y)
        }).sorted_by_key(|cp| cp.0).collect::<Vec<_>>();

        // Only add twigs if first and last main branch both exist
        let first = choices_and_positions.first();
        let last = choices_and_positions.last();
        if let (
            Some(&(first_choice, first_x, first_y)),
            Some(&(last_choice, last_x, last_y))
        ) = (first, last) {

            // Grow the twigs before the first branch with fixed angle
            let twig_first_x = (1.0 - TWIG_LENGTH_SCALE) * branch_point_x + TWIG_LENGTH_SCALE * first_x;
            let twig_first_y = (1.0 - TWIG_LENGTH_SCALE) * branch_point_y + TWIG_LENGTH_SCALE * first_y;
            for branch_choice in 0..first_choice {
                decorate_branch_choice_to_fractal_fan(
                    graph, branch_point_idx, base, direction, twig_depth,
                    &FractalFanDisplay {
                        unscaled_twig_width: branch_width / f64::from(base),
                        unscaled_twig_length: branch_length,
                        branch_choice, reference_choice: first_choice,
                        ref_x: twig_first_x, ref_y: twig_first_y,
                    },
                    path_group,
                )?;
            }

            // Grow the twigs between each main branches with even spacing
            for (prev_main, next_main) in choices_and_positions.into_iter().tuple_windows() {

                let (prev_choice, prev_x, prev_y) = prev_main;
                let (next_choice, next_x, next_y) = next_main;
                let twig_prev_x = (1.0 - TWIG_LENGTH_SCALE) * branch_point_x + TWIG_LENGTH_SCALE * prev_x;
                let twig_prev_y = (1.0 - TWIG_LENGTH_SCALE) * branch_point_y + TWIG_LENGTH_SCALE * prev_y;
                let even_spaced_x = (next_x - prev_x).abs() / f64::from(next_choice - prev_choice);
                let even_spaced_y = (next_y - prev_y).abs() / f64::from(next_choice - prev_choice);
                let even_spaced_width = match direction {
                    Up | Down => even_spaced_x,
                    Left | Right => even_spaced_y,
                };
                for branch_choice in prev_choice+1..next_choice {
                    decorate_branch_choice_to_fractal_fan(
                        graph, branch_point_idx, base, direction, twig_depth,
                        &FractalFanDisplay {
                            unscaled_twig_width: even_spaced_width,
                            unscaled_twig_length: branch_length,
                            branch_choice, reference_choice: prev_choice,
                            ref_x: twig_prev_x, ref_y: twig_prev_y,
                        },
                        path_group,
                    )?;
                }

            }

            // Grow the twigs after the last branch with fixed angle
            let twig_last_x = (1.0 - TWIG_LENGTH_SCALE) * branch_point_x + TWIG_LENGTH_SCALE * last_x;
            let twig_last_y = (1.0 - TWIG_LENGTH_SCALE) * branch_point_y + TWIG_LENGTH_SCALE * last_y;
            for branch_choice in last_choice+1..base {
                decorate_branch_choice_to_fractal_fan(
                    graph, branch_point_idx, base, direction, twig_depth,
                    &FractalFanDisplay {
                        unscaled_twig_width: branch_width / f64::from(base),
                        unscaled_twig_length: branch_length,
                        branch_choice, reference_choice: last_choice,
                        ref_x: twig_last_x, ref_y: twig_last_y,
                    },
                    path_group,
                )?;
            }

        }

    }

    Ok(())

}

struct FractalFanDisplay {
    unscaled_twig_width: f64,
    unscaled_twig_length: f64,
    branch_choice: u32,
    reference_choice: u32,
    ref_x: f64,
    ref_y: f64,
}

fn decorate_branch_choice_to_fractal_fan(
    graph: &mut TreeGraph, branch_point_idx: NodeIndex, base: u32,
    direction: Direction, twig_depth: u32, fan_disp: &FractalFanDisplay,
    path_group: PathGroup,
) -> AdicShapeResult<()> {

    let bp = &graph[branch_point_idx];
    let twig_diff = f64::from(fan_disp.branch_choice) - f64::from(fan_disp.reference_choice);
    let twig_width = twig_diff * fan_disp.unscaled_twig_width * TWIG_WIDTH_SCALE;
    let twig_length = fan_disp.unscaled_twig_length * TWIG_LENGTH_SCALE;
    let (twig_x, twig_y) = match direction {
        Up => (fan_disp.ref_x + twig_width, bp.y - twig_length),
        Down => (fan_disp.ref_x - twig_width, bp.y + twig_length),
        Left => (bp.x - twig_length, fan_disp.ref_y - twig_width),
        Right => (bp.x + twig_length, fan_disp.ref_y + twig_width),
    };

    let (leaf_idx, _twig_idx) = add_branch_from(
        graph, branch_point_idx, fan_disp.branch_choice,
        twig_x, twig_y, TWIG_WIDTH_SCALE, TWIG_LENGTH_SCALE,
        path_group,
    )?;

    decorate_fractal_fan(graph, leaf_idx, base, direction, twig_depth - 1, path_group)?;

    Ok(())

}

// Optionally add sub twigs
fn decorate_fractal_fan(
    graph: &mut TreeGraph, branch_point_idx: NodeIndex, base: u32,
    direction: Direction, twig_depth: u32, path_group: PathGroup,
) -> AdicShapeResult<()> {

    if base.pow(twig_depth) > 1000 {
        Err(AdicShapeError::TooLarge("Twig depth too large; exponential branch numbers".to_string()))?;
    }

    let mut bps = vec![branch_point_idx];
    for _ in 0..twig_depth {
        let mut new_bps = vec![];
        for bp_idx in bps {

            let TreeNode { x: bx, y: by, branch_width: bw, branch_length: bl, .. } = graph[bp_idx];

            for sub_choice in 0..base {
                let twig_wdiff = (f64::from(sub_choice) + 0.5) / f64::from(base) - 0.5;
                let twig_wdiff = twig_wdiff * bw / f64::from(base);
                let twig_ldiff = bl * TWIG_LENGTH_SCALE;
                let (twig_x, twig_y) = match direction {
                    Up => (bx - twig_wdiff, by - twig_ldiff),
                    Down => (bx + twig_wdiff, by + twig_ldiff),
                    Left => (bx - twig_ldiff, by + twig_wdiff),
                    Right => (bx + twig_ldiff, by - twig_wdiff),
                };
                let (leaf_idx, _twig_idx) = add_branch_from(
                    graph, bp_idx, sub_choice,
                    twig_x, twig_y, 1.0 / f64::from(base), TWIG_LENGTH_SCALE,
                    path_group,
                )?;
                new_bps.push(leaf_idx);
            }

        }
        bps = new_bps;
    }

    Ok(())

}


/// Calculate `reg_length` and `root_length` of tree
///
/// # Errors
/// Errors for integer conversion failure
pub (super) fn calc_branch_lengths(
    base: u32, num_levels: usize, has_dangling: bool,
) -> AdicShapeResult<(f64, f64)> {
    let f64_levels = f64::from(u32::try_from(num_levels)?);
    if has_dangling {
        // Root has a larger angle, smaller length
        // root_length * (p - 1)/2 = reg_length
        // root_length + reg_length * num_levels = 1
        // root_length * (1 + (p - 1) / 2 * num_levels) = 1
        let root_length = 2.  / (2. + (f64::from(base) - 1.) * f64_levels);
        let reg_length = (1. - root_length) / f64_levels;
        Ok((root_length, reg_length))
    } else {
        let root_length = 0.;
        let reg_length = 1. / f64_levels;
        Ok((root_length, reg_length))
    }
}

/// Add root node and possibly dangling node if `dangling_direction` is given
///
/// # Errors
/// Error if `root_direction` is inconsistent with `direction`, i.e. in the SAME direction
fn root_tree(
    graph: &mut TreeGraph, direction: Direction, dangling_direction: Option<Direction>,
    path_group: PathGroup,
    branch_length: f64, root_length: f64,
) -> AdicShapeResult<(NodeIndex, Option<NodeIndex>)> {

    if let Some(root_direction) = dangling_direction {

        let (x1, y1, x2, y2) = root_node_pos(direction, root_direction, root_length)?;

        let some_dangling_idx = graph.add_node(TreeNode {
            x: x1, y: y1, depth: -1, branch_width: 1.0, branch_length: root_length,
        });
        let dangling_idx = Some(some_dangling_idx);
        let root_idx = graph.add_node(TreeNode{
            x: x2, y: y2, depth: 0, branch_width: 1.0, branch_length,
        });
        graph.add_edge(some_dangling_idx, root_idx, TreeEdge{
            branch_choice: 0,
            path_group,
        });
        Ok((root_idx, dangling_idx))

    } else {

        // With no root, just start from the center of the left side
        let (x, y) = match direction {
            Up => (0.5, 1.),
            Down => (0.5, 0.),
            Left => (1., 0.5),
            Right => (0., 0.5),
        };
        let root_idx = graph.add_node(TreeNode{
            x, y, depth: 0, branch_width: 1.0, branch_length,
        });
        Ok((root_idx, None))

    }

}

/// Add root node with total width and `root_length` height,
///  pointing straight to the actual start of the tree.
///
/// # Errors
/// Error if `root_direction` is inconsistent with `direction`, i.e. in the SAME direction
fn root_node_pos(
    direction: Direction, root_direction: Direction, root_length: f64
) -> AdicShapeResult<(f64, f64, f64, f64)> {
    // Note this is inconsistent with the above calculation of angle; width and height would be larger.
    // But these probably work together just fine, since this first node will not really be used.
    let pos = match (direction, root_direction) {
        (Up, Left) => (0., 1., 0.5, 1. - root_length),
        (Up, Down) => (0.5, 1., 0.5, 1. - root_length),
        (Up, Right) => (1., 1., 0.5, 1. - root_length),
        (Down, Left) => (0., 0., 0.5, root_length),
        (Down, Up) => (0.5, 0., 0.5, root_length),
        (Down, Right) => (1., 0., 0.5, root_length),
        (Left, Up) => (1., 0., 1. - root_length, 0.5),
        (Left, Right) => (1., 0.5, 1. - root_length, 0.5),
        (Left, Down) => (1., 1., 1. - root_length, 0.5),
        (Right, Up) => (0., 0., root_length, 0.5),
        (Right, Left) => (0., 0.5, root_length, 0.5),
        (Right, Down) => (0., 1., root_length, 0.5),
        _ => {
            return Err(AdicShapeError::ImproperConfig(
                "Cannot have a dangling root in the same direction as the tree's growth".to_string()
            ));
        }
    };
    Ok(pos)
}


/// Add a branch to `graph` with branch num `p`
/// Branch goes from `origin_idx` to a new node at `(endpoint_x, endpoint_y)`
///
/// # Errors
/// Errors if `graph` cannot find `origin_idx`
#[allow(clippy::too_many_arguments)]
fn add_branch_from(
    graph: &mut TreeGraph, origin_idx: NodeIndex, branch_choice: u32,
    endpoint_x: f64, endpoint_y: f64, width_scale: f64, length_scale: f64,
    path_group: PathGroup,
) -> AdicShapeResult<(NodeIndex, EdgeIndex)> {

    let origin_bp = graph.node_weight(origin_idx).ok_or(AdicShapeError::PetGraph)?;
    let new_width = origin_bp.branch_width * width_scale;
    let new_length = origin_bp.branch_length * length_scale;

    // Add node and edge from branch point to the possibility
    let new_idx = graph.add_node(TreeNode{
        x: endpoint_x, y: endpoint_y,
        depth: origin_bp.depth + 1, branch_width: new_width, branch_length: new_length,
    });
    let edge_idx = graph.add_edge(origin_idx, new_idx, TreeEdge {
        branch_choice,
        path_group,
    });
    Ok((new_idx, edge_idx))

}
