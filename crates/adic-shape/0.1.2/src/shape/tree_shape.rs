use std::iter::{once, repeat};
use adic::AdicInteger;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::{EdgeRef, NodeRef},
    Directed, Graph
};

use crate::{AdicShapeError, DisplayShape};
use super::{
    element::{AdicEl, PathDInstruction, PathEl},
    Direction,
};


/// List of branch choices that should be drawn in gold
type GoldBranchChoices = Vec<u32>;
type Coordinate = (f64, f64);

use Direction::{Left, Right, Up, Down};


pub type TreeGraph = Graph<TreeNode, TreeEdge, Directed>;


#[derive(Debug, Clone)]
/// Tree shape for drawing
///
/// ```
/// # use adic::radic;
/// # use adic_shape::{Direction, TreeShape, TreeShapeOptions};
/// let a = radic!(5, [1, 2, 3, 4], [0, 3]);
/// let depth = 20;
/// let shape_options = TreeShapeOptions{
///     direction: Direction::Up,
///     ..Default::default()
/// };
/// let tree_shape = TreeShape::zoomed_tree(&a, depth, shape_options);
/// ```
#[doc = ""]
#[doc = "<style>"]
#[doc = include_str!("../../img/rustdoc.css")]
#[doc = "</style>"]
#[doc = ""]
#[doc = include_str!("../../img/tree-shape-example.svg")]
#[doc = ""]
pub struct TreeShape {
    /// Number of ticks on the clock
    p: u32,
    /// Options for creating `TreeShape`
    options: TreeShapeOptions,
    /// Tree position
    position: TreePosition,
    /// Stores the tree information
    graph: TreeGraph,
    /// Root node index of the tree
    root_idx: NodeIndex<u32>,
    /// Index for node dangling from root node of the tree
    dangling_idx: Option<NodeIndex<u32>>,
    /// Special branches colored differently, e.g. to indicate a path through the tree
    gold_branches: Vec<GoldBranchChoices>,
}


#[derive(Debug, Clone, Copy)]
/// Options for creating tree shape
pub struct TreeShapeOptions {
    /// Direction the tree is growing
    pub direction: Direction,
    /// Direction that node is dangling from root node of the tree
    pub dangling_direction: Option<Direction>,
    /// Width of the tree window
    pub viewbox_width: u32,
    /// Height of the tree window
    pub viewbox_height: u32,
}

impl Default for TreeShapeOptions {
    fn default() -> Self {
        TreeShapeOptions {
            direction: Up,
            dangling_direction: Some(Right),
            viewbox_width: 100,
            viewbox_height: 100,
        }
    }
}


#[derive(Debug, Clone, Copy)]
/// Tree layout information
pub struct TreePosition {
    /// Position of center of the tree
    pub center: Coordinate,
}


impl TreeShape {

    /// `TreeShape` showing a full tree with no special branches
    ///
    /// # Errors
    /// Errors for (1) integer conversion failure, (2) inconsistent directions, and (3) petgraph inconsistency
    ///
    /// <div class="warning">
    ///
    /// This full tree uses a LOT of lines if the depth is large, on the order of `p^depth`.
    /// Consider using [`zoomed_tree`](Self::zoomed_tree) instead.
    ///
    /// </div>
    pub fn full_tree(
        p: u32,
        depth: usize,
        options: TreeShapeOptions,
    ) -> Result<Self, AdicShapeError> {

        let TreeShapeOptions { direction, dangling_direction, .. } = options;

        let (mut graph, root_idx, dangling_idx) = create_tree_graph(p, depth, direction, dangling_direction)?;

        let width = f64::from(options.viewbox_width);
        let height = f64::from(options.viewbox_height);
        adjust_size(&mut graph, direction, width, height);

        Ok(Self {
            p,
            position: TreePosition {
                center: (0.5*width, 0.5*height),
            },
            graph,
            root_idx,
            dangling_idx,
            options,
            gold_branches: vec![],
        })

    }

    /// `TreeShape` showing a full tree with a gold branch showing `adic_data`
    ///
    /// # Errors
    /// Errors for (1) integer conversion failure, (2) inconsistent directions, and (3) petgraph inconsistency
    ///
    /// <div class="warning">
    ///
    /// This full tree uses a LOT of lines if the depth is large, on the order of `p^depth`.
    /// Consider using [`zoomed_tree`](Self::zoomed_tree) instead.
    ///
    /// </div>
    pub fn adic_number_full_tree(
        adic_data: &impl AdicInteger,
        depth: usize,
        options: TreeShapeOptions,
    ) -> Result<Self, AdicShapeError> {

        let p = adic_data.p();
        let TreeShapeOptions { direction, dangling_direction, .. } = options;

        let (mut graph, root_idx, dangling_idx) = create_tree_graph(p, depth, direction, dangling_direction)?;

        let width = f64::from(options.viewbox_width);
        let height = f64::from(options.viewbox_height);
        adjust_size(&mut graph, direction, width, height);

        Ok(Self {
            p,
            position: TreePosition {
                center: (0.5*width, 0.5*height),
            },
            graph,
            root_idx,
            dangling_idx,
            options,
            gold_branches: vec![adic_data.digits().take(depth).copied().collect()]
        })

    }

    /// `TreeShape` showing a zoomed tree, only fully following the `adic_data` branch choice
    ///
    /// # Errors
    /// Errors for (1) integer conversion failure, (2) inconsistent directions, and (3) petgraph inconsistency
    pub fn zoomed_tree(
        adic_data: &impl AdicInteger,
        depth: usize,
        options: TreeShapeOptions,
    ) -> Result<Self, AdicShapeError> {

        let p = adic_data.p();
        let TreeShapeOptions { direction, dangling_direction, .. } = options;

        let (mut graph, root_idx, dangling_idx) = create_zoomed_tree_graph(adic_data.clone(), depth, direction, dangling_direction)?;

        let width = f64::from(options.viewbox_width);
        let height = f64::from(options.viewbox_height);
        adjust_size(&mut graph, direction, width, height);

        Ok(Self {
            p,
            position: TreePosition {
                center: (0.5*width, 0.5*height),
            },
            graph,
            root_idx,
            dangling_idx,
            options,
            gold_branches: vec![adic_data.digits().take(depth).copied().collect()]
        })

    }


    /// Number of branches from each branch point
    pub fn p(&self) -> u32 {
        self.p
    }

    /// Centerpoint for the tree
    pub fn center(&self) -> (f64, f64) {
        self.position.center
    }

    /// Path instructions (Move and Line) to draw the base tree
    ///
    /// # Errors
    /// Errors if graph gets into a bad state
    fn tree_instructions(&self) -> Result<Vec<PathDInstruction>, AdicShapeError> {

        let mut instructions = vec![];
        for e in self.graph.edge_references() {
            let source = self.graph.node_weight(e.source()).ok_or(AdicShapeError::PetGraph)?;
            let target = self.graph.node_weight(e.target()).ok_or(AdicShapeError::PetGraph)?;
            instructions.push(PathDInstruction::Move((source.x, source.y)));
            instructions.push(PathDInstruction::Line((target.x, target.y)));
        }

        Ok(instructions)

    }


    /// Path instructions (Move and Line) to draw the gold branches
    ///
    /// # Errors
    /// Errors if graph gets into a bad state
    fn gold_branch_instructions(&self) -> Result<Vec<Vec<PathDInstruction>>, AdicShapeError> {

        let mut all_instructions = vec![];
        for gold_branch_choices in &self.gold_branches {

            let (mut node_id, choices) = if let Some(dangling_idx) = self.dangling_idx {
                let mut choices = vec![0];
                choices.extend(gold_branch_choices);
                (dangling_idx, choices)
            } else {
                (self.root_idx, gold_branch_choices.clone())
            };

            let mut instructions = vec![];
            for gold_choice in choices.into_iter().chain(repeat(0)) {
                let mut edges = self.graph.edges(node_id);
                let gold_edge = edges.find(|e| e.weight().branch_choice == gold_choice);
                if let Some(ge) = gold_edge {
                    node_id = ge.target().id();
                    let source = self.graph.node_weight(ge.source()).ok_or(AdicShapeError::PetGraph)?;
                    let target = self.graph.node_weight(ge.target()).ok_or(AdicShapeError::PetGraph)?;
                    instructions.push(PathDInstruction::Move((source.x, source.y)));
                    instructions.push(PathDInstruction::Line((target.x, target.y)));
                } else {
                    break;
                }
            }
            all_instructions.push(instructions);

        }

        Ok(all_instructions)

    }

}

impl DisplayShape for TreeShape {

    /// Internal SVG elements generated from this shape
    fn adic_els(&self) -> impl Iterator<Item=AdicEl> {

        // Draw the tree
        let tree_ins = self.tree_instructions().expect("tree instructions should succeed");
        let tree_path = AdicEl::Path(PathEl{
            class: Some("tree-path".to_string()),
            d: tree_ins
        });
        // Draw the labels
        // let labels = labeller.labels(&tree_diagram, num_depth, num_branch, Default::default());

        // Draw the gold paths
        let gold_paths = self.gold_branch_instructions()
            .expect("gold branch instructions should succeed")
            .into_iter()
            .map(
                |gold_ins| AdicEl::Path(PathEl{
                    class: Some("gold-path".to_string()),
                    d: gold_ins
                })
            )
            .collect::<Vec<_>>();

        // Wrap in svg
        once(tree_path)
            // .chain(labels)
            .chain(gold_paths)

    }

    fn default_class(&self) -> String {
        "adic-tree".to_string()
    }

    fn viewbox_width(&self) -> u32 {
        self.options.viewbox_width
    }
    fn viewbox_height(&self) -> u32 {
        self.options.viewbox_height
    }
}


#[derive(Debug, Clone)]
/// Data for each tree node, i.e. branch splitting
pub struct TreeNode {
    /// x-value for node
    pub x: f64,
    /// y-value for node
    pub y: f64,
    /// Depth into the directed tree, i.e. displacement from root node
    pub depth: isize,
}

#[derive(Debug, Clone)]
/// Data for each tree edge, i.e. branch
pub struct TreeEdge {
    /// Which branch is selected, going from the source to target
    pub branch_choice: u32,
}


/// Adjust x & y for width and height
fn adjust_size(graph: &mut TreeGraph, direction: Direction, width: f64, height: f64) {
    match direction {
        Up | Down => {
            graph.node_weights_mut().for_each(|n| {
                n.x = width * n.x;
                n.y = height * n.y;
            });
        },
        Left | Right => {
            graph.node_weights_mut().for_each(|n| {
                n.x = width * n.x;
                n.y = height * n.y;
            });
        },
    }
}


/// Create a "full tree graph", plotting the ENTIRE tree of possible adic numbers
///
/// # Errors
/// Errors for (1) integer conversion failure, (2) inconsistent directions, and (3) petgraph inconsistency
fn create_tree_graph(
    p: u32,
    depth: usize,
    direction: Direction,
    dangling_direction: Option<Direction>
) -> Result<(TreeGraph, NodeIndex, Option<NodeIndex>), AdicShapeError> {

    let mut graph = Graph::new();
    let (root_length, reg_length) = calc_branch_lengths(p, depth, dangling_direction)?;
    let (root_idx, dangling_idx) = root_tree(&mut graph, direction, dangling_direction, root_length)?;

    let mut cur_branch_point = root_idx;
    let mut cur_branch_choice = 0;
    let mut cur_width = 1.;
    let mut cur_node = graph.node_weight(cur_branch_point).ok_or(AdicShapeError::PetGraph)?;
    let mut cur_edge_set: Vec<EdgeIndex> = vec![];
    'tree_pos_loop: loop {

        // If current depth is less than max, go deeper
        if cur_node.depth < depth.try_into()? {

            let adjust_choice = ((f64::from(cur_branch_choice) + 0.5) / f64::from(p) - 0.5) * cur_width;
            let adjust_depth = reg_length;

            let (new_idx, edge_idx) = add_relative_branch(
                &mut graph, cur_branch_point, cur_branch_choice,
                direction, adjust_choice, adjust_depth,
            )?;

            cur_branch_point = new_idx;
            cur_branch_choice = 0;
            cur_width = cur_width / f64::from(p);
            cur_node = graph.node_weight(new_idx).ok_or(AdicShapeError::PetGraph)?;
            cur_edge_set.push(edge_idx);

        }
        // Else, pop up until the next branch choice is available or we're done with the graph
        else {

            while (cur_node.depth >= depth.try_into()?) || (cur_branch_choice == p - 1) {

                // If we're at the root, we're done
                if cur_node.depth == 0 {
                    break 'tree_pos_loop;
                }

                let old_edge_idx = cur_edge_set.pop().ok_or(AdicShapeError::PetGraph)?;
                cur_branch_point = graph.edge_endpoints(old_edge_idx).ok_or(AdicShapeError::PetGraph)?.0;
                cur_branch_choice = graph.edge_weight(old_edge_idx).ok_or(AdicShapeError::PetGraph)?.branch_choice;
                cur_width = cur_width * f64::from(p);

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
fn create_zoomed_tree_graph(
    adic_data: impl AdicInteger,
    depth: usize,
    direction: Direction,
    dangling_direction: Option<Direction>
) -> Result<(TreeGraph, NodeIndex, Option<NodeIndex>), AdicShapeError> {

    let p = adic_data.p();

    let mut graph = Graph::new();
    let (root_length, reg_length) = calc_branch_lengths(p, depth, dangling_direction)?;
    let (root_idx, dangling_idx) = root_tree(&mut graph, direction, dangling_direction, root_length)?;

    let mut next_branch_point = root_idx;
    let mut next_choice_pos = 0.5;
    let choices = adic_data.into_digits().chain(repeat(0)).take(depth-1);
    for (num_digit, choice) in choices.into_iter().enumerate() {

        let cur_branch_point = next_branch_point;
        let cur_choice_pos = next_choice_pos;
        next_choice_pos = (f64::from(choice) + 0.5) / f64::from(p);
        let branch_width = 1.;
        let branch_length = reg_length;
        let unchosen_width_scale = 0.4;
        let unchosen_length_scale = 0.6;
        let twig_width_scale = 0.05;
        let twig_length_scale = 0.2;

        // Draw branches from current branch point up for each possible choice
        for branch in (0..p) {

            // Add node and edge from branch point to the possibility
            let new_width =  branch_width * if branch == choice { 1. } else { unchosen_width_scale };
            let new_length = branch_length * if branch == choice { 1. } else { unchosen_length_scale };

            let adjust_choice = ((f64::from(branch) + 0.5) / f64::from(p) - 0.5) * new_width;
            let adjust_choice = adjust_choice + if branch == choice {
                (0.5 - cur_choice_pos)
            } else {
                let main_branch_adjust = unchosen_length_scale * (next_choice_pos - cur_choice_pos);
                let choice_adjust = (0.5 * f64::from(p-1) - f64::from(choice)) / f64::from(p) * branch_width * unchosen_width_scale;
                main_branch_adjust + choice_adjust
            };
            let adjust_depth = new_length;

            let (next_idx, _edge_idx) = add_relative_branch(
                &mut graph, cur_branch_point, branch,
                direction, adjust_choice, adjust_depth,
            )?;

            // Add short branches for every sub-choice if it's not the RIGHT choice
            if branch == choice {

                if num_digit + 2 == depth {
                    // Add one more fan of short branches for last choice
                    for twig_choice in (0..p) {
                        let final_choice_pos = cur_choice_pos;
                        let twig_width = branch_width * twig_width_scale;
                        let adjust_choice = ((f64::from(twig_choice) + 0.5) / f64::from(p) - 0.5) * twig_width;
                        let adjust_choice = adjust_choice + twig_width * (0.5 - final_choice_pos);
                        let adjust_depth = branch_length * twig_length_scale;
                        let (_leaf_idx, _twig_idx) = add_relative_branch(
                            &mut graph, next_idx, twig_choice,
                            direction, adjust_choice, adjust_depth,
                        )?;
                    }
                } else {
                    next_branch_point = next_idx;
                }

            } else {

                for twig_choice in (0..p) {

                    let adjust_choice = ((f64::from(twig_choice) + 0.5) / f64::from(p) - 0.5) * branch_width * twig_width_scale;
                    let adjust_depth = branch_length * twig_length_scale;
                    let (_leaf_idx, _twig_idx) = add_relative_branch(
                        &mut graph, next_idx, twig_choice,
                        direction, adjust_choice, adjust_depth,
                    )?;

                }

            }

        }

    }

    Ok((graph, root_idx, dangling_idx))

}


/// Calculate `reg_length` and `root_length` of tree
///
/// # Errors
/// Errors for integer conversion failure
fn calc_branch_lengths(
    p: u32, num_levels: usize, dangling_direction: Option<Direction>
) -> Result<(f64, f64), AdicShapeError> {
    let f64_levels = f64::from(u32::try_from(num_levels)?);
    if dangling_direction.is_some() {
        // Root has a larger angle, smaller length
        // root_length * (p - 1)/2 = reg_length
        // root_length + reg_length * num_levels = 1
        // root_length * (1 + (p - 1) / 2 * num_levels) = 1
        let root_length = 2.  / (2. + (f64::from(p) - 1.) * f64_levels);
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
    root_length: f64,
) -> Result<(NodeIndex, Option<NodeIndex>), AdicShapeError> {

    if let Some(root_direction) = dangling_direction {

        let (x1, y1, x2, y2) = root_node_pos(direction, root_direction, root_length)?;

        let some_dangling_idx = graph.add_node(TreeNode {
            x: x1, y: y1, depth: -1,
        });
        let dangling_idx = Some(some_dangling_idx);
        let root_idx = graph.add_node(TreeNode{
            x: x2, y: y2, depth: 0,
        });
        graph.add_edge(some_dangling_idx, root_idx, TreeEdge{
            branch_choice: 0,
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
            x, y, depth: 0,
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
) -> Result<(f64, f64, f64, f64), AdicShapeError> {
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
/// Relative to `origin_idx`, fanning out in `direction` with `new_width` fan width and `new_length` fan length
/// Adjust fan width center by `adjust_choice`
///
/// # Errors
/// Errors if `graph` cannot find `origin_idx`
fn add_relative_branch(
    graph: &mut TreeGraph, origin_idx: NodeIndex, branch_choice: u32, direction: Direction,
    adjust_choice: f64, adjust_depth: f64,
) -> Result<(NodeIndex, EdgeIndex), AdicShapeError> {
    let origin_bp = graph.node_weight(origin_idx).ok_or(AdicShapeError::PetGraph)?;
    let (new_x, new_y) = match direction {
        Up => (origin_bp.x + adjust_choice, origin_bp.y - adjust_depth),
        Down => (origin_bp.x - adjust_choice, origin_bp.y + adjust_depth),
        Left => (origin_bp.x - adjust_depth, origin_bp.y - adjust_choice),
        Right => (origin_bp.x + adjust_depth, origin_bp.y + adjust_choice),
    };

    // Add node and edge from branch point to the possibility
    let new_idx = graph.add_node(TreeNode{
        x: new_x, y: new_y,
        depth: origin_bp.depth + 1,
    });
    let edge_idx = graph.add_edge(origin_idx, new_idx, TreeEdge {
        branch_choice,
    });
    Ok((new_idx, edge_idx))
}



#[cfg(test)]
mod test {
    use adic::uadic;
    use super::{TreeShape, TreeShapeOptions};

    #[test]
    fn correct_numbers() {

        let shape_options = TreeShapeOptions::default();

        let tree_shape = TreeShape::full_tree(5, 6, shape_options).unwrap();
        assert_eq!(5, tree_shape.p);
        assert_eq!(0, tree_shape.gold_branches.len());

        let adic_data = uadic!(5, [3, 2, 4, 1, 4, 1, 2]);
        let tree_shape = TreeShape::adic_number_full_tree(&adic_data, 6, shape_options).unwrap();
        assert_eq!(5, tree_shape.p);
        assert_eq!(1, tree_shape.gold_branches.len());

        let gold_branch = &tree_shape.gold_branches[0];
        assert_eq!(6, gold_branch.len());
        assert_eq!(3, gold_branch[0]);
        assert_eq!(2, gold_branch[1]);
        assert_eq!(4, gold_branch[2]);
        assert_eq!(1, gold_branch[3]);
        assert_eq!(4, gold_branch[4]);
        assert_eq!(1, gold_branch[5]);

        let adic_data = uadic!(5, [3, 2, 4, 1, 4, 1, 2]);
        let tree_shape = TreeShape::zoomed_tree(&adic_data, 6, shape_options).unwrap();
        assert_eq!(5, tree_shape.p);
        assert_eq!(1, tree_shape.gold_branches.len());

        let gold_branch = &tree_shape.gold_branches[0];
        assert_eq!(6, gold_branch.len());
        assert_eq!(3, gold_branch[0]);
        assert_eq!(2, gold_branch[1]);
        assert_eq!(4, gold_branch[2]);
        assert_eq!(1, gold_branch[3]);
        assert_eq!(4, gold_branch[4]);
        assert_eq!(1, gold_branch[5]);

    }

    #[test]
    fn correct_bounds() {

        let shape_options = TreeShapeOptions::default();

        let tree_shape = TreeShape::full_tree(5, 6, shape_options).unwrap();
        assert!(0. <= tree_shape.graph.node_weights().map(|nw| nw.x).min_by(f64::total_cmp).unwrap());
        assert!(100. >= tree_shape.graph.node_weights().map(|nw| nw.x).max_by(f64::total_cmp).unwrap());
        assert!(0. <= tree_shape.graph.node_weights().map(|nw| nw.y).min_by(f64::total_cmp).unwrap());
        assert!(100. >= tree_shape.graph.node_weights().map(|nw| nw.y).max_by(f64::total_cmp).unwrap());

        let adic_data = uadic!(5, [3, 2, 4, 1, 4, 1, 2]);
        let tree_shape = TreeShape::adic_number_full_tree(&adic_data, 6, shape_options).unwrap();
        assert!(0. <= tree_shape.graph.node_weights().map(|nw| nw.x).min_by(f64::total_cmp).unwrap());
        assert!(100. >= tree_shape.graph.node_weights().map(|nw| nw.x).max_by(f64::total_cmp).unwrap());
        assert!(0. <= tree_shape.graph.node_weights().map(|nw| nw.y).min_by(f64::total_cmp).unwrap());
        assert!(100. >= tree_shape.graph.node_weights().map(|nw| nw.y).max_by(f64::total_cmp).unwrap());

        let adic_data = uadic!(5, [3, 2, 4, 1, 4, 1, 2]);
        let tree_shape = TreeShape::zoomed_tree(&adic_data, 6, shape_options).unwrap();
        assert!(0. <= tree_shape.graph.node_weights().map(|nw| nw.x).min_by(f64::total_cmp).unwrap());
        assert!(100. >= tree_shape.graph.node_weights().map(|nw| nw.x).max_by(f64::total_cmp).unwrap());
        assert!(0. <= tree_shape.graph.node_weights().map(|nw| nw.y).min_by(f64::total_cmp).unwrap());
        assert!(100. >= tree_shape.graph.node_weights().map(|nw| nw.y).max_by(f64::total_cmp).unwrap());

    }

    #[test]
    fn graph_stats() {

        let shape_options = TreeShapeOptions::default();

        let tree_shape = TreeShape::full_tree(5, 4, shape_options).unwrap();
        let expected_num_branches = 625 + 125 + 25 + 5 + 1;
        assert_eq!(expected_num_branches + 1, tree_shape.graph.node_count());
        assert_eq!(expected_num_branches, tree_shape.graph.edge_count());
        assert_eq!(625 + 125 + 25 + 5 + 1 + 1, tree_shape.graph.node_count());
        assert_eq!(625 + 125 + 25 + 5 + 1, tree_shape.graph.edge_count());
        assert_eq!(1, tree_shape.graph.node_weights().filter(|nw| nw.depth == -1).count());
        assert_eq!(1, tree_shape.graph.node_weights().filter(|nw| nw.depth == 0).count());
        assert_eq!(5, tree_shape.graph.node_weights().filter(|nw| nw.depth == 1).count());
        assert_eq!(25, tree_shape.graph.node_weights().filter(|nw| nw.depth == 2).count());
        assert_eq!(125, tree_shape.graph.node_weights().filter(|nw| nw.depth == 3).count());
        assert_eq!(625, tree_shape.graph.node_weights().filter(|nw| nw.depth == 4).count());
        assert_eq!((expected_num_branches-1)/5 + 1, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 0).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 1).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 2).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 3).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 4).count());

        let adic_data = uadic!(5, [3, 2, 4, 1, 4, 1, 2]);
        let tree_shape = TreeShape::adic_number_full_tree(&adic_data, 4, shape_options).unwrap();
        let expected_num_branches = 625 + 125 + 25 + 5 + 1;
        assert_eq!(expected_num_branches + 1, tree_shape.graph.node_count());
        assert_eq!(expected_num_branches, tree_shape.graph.edge_count());
        assert_eq!(1, tree_shape.graph.node_weights().filter(|nw| nw.depth == -1).count());
        assert_eq!(1, tree_shape.graph.node_weights().filter(|nw| nw.depth == 0).count());
        assert_eq!(5, tree_shape.graph.node_weights().filter(|nw| nw.depth == 1).count());
        assert_eq!(25, tree_shape.graph.node_weights().filter(|nw| nw.depth == 2).count());
        assert_eq!(125, tree_shape.graph.node_weights().filter(|nw| nw.depth == 3).count());
        assert_eq!(625, tree_shape.graph.node_weights().filter(|nw| nw.depth == 4).count());
        assert_eq!((expected_num_branches-1)/5 + 1, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 0).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 1).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 2).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 3).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 4).count());

        let adic_data = uadic!(5, [3, 2, 4, 1, 4, 1, 2]);
        let tree_shape = TreeShape::zoomed_tree(&adic_data, 4, shape_options).unwrap();
        let expected_num_branches = 5 + 20 + 5 + 20 + 5 + 20 + 5 + 1;
        assert_eq!(expected_num_branches + 1, tree_shape.graph.node_count());
        assert_eq!(expected_num_branches, tree_shape.graph.edge_count());
        assert_eq!(1, tree_shape.graph.node_weights().filter(|nw| nw.depth == -1).count());
        assert_eq!(1, tree_shape.graph.node_weights().filter(|nw| nw.depth == 0).count());
        assert_eq!(5, tree_shape.graph.node_weights().filter(|nw| nw.depth == 1).count());
        assert_eq!(20 + 5, tree_shape.graph.node_weights().filter(|nw| nw.depth == 2).count());
        assert_eq!(20 + 5, tree_shape.graph.node_weights().filter(|nw| nw.depth == 3).count());
        assert_eq!(20 + 5, tree_shape.graph.node_weights().filter(|nw| nw.depth == 4).count());
        assert_eq!((expected_num_branches-1)/5 + 1, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 0).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 1).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 2).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 3).count());
        assert_eq!((expected_num_branches-1)/5, tree_shape.graph.edge_weights().filter(|ew| ew.branch_choice == 4).count());

    }

}
