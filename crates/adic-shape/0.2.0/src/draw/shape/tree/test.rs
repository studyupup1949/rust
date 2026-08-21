use assertables::assert_matches;
use adic::EAdic;
use crate::{
    error::{AdicShapeError, AdicShapeResult},
    shape::AdicCanvas,
};
use super::{
    graph::{TreeBranch, TreeEdge, TreeGraph},
    TreeCanvas,
};


#[test]
fn correct_numbers() {

    fn edge_weights(branch: &TreeBranch, graph: &TreeGraph) -> AdicShapeResult<Vec<TreeEdge>> {
        branch.0.iter().map(|ei| graph.edge_weight(*ei).cloned().ok_or(AdicShapeError::PetGraph)).collect()
    }

    let tree_shape = TreeCanvas::builder().base(5).depth(6).solid_full_tree().build().draw_full().unwrap();
    assert_eq!(5, tree_shape.base());
    assert_eq!(0, tree_shape.colored_branches().count());

    let adic = EAdic::new(5, vec![3, 2, 4, 1, 4, 1, 2]);
    let tree_shape = TreeCanvas::builder().base(5).depth(6).solid_full_tree().build().draw_integer(&adic).unwrap();
    assert_eq!(5, tree_shape.base());
    assert_eq!(1, tree_shape.colored_branches().count());

    let colored_branch = tree_shape.colored_branches().next().unwrap();
    assert_eq!(7, colored_branch.0.len());
    let branch_weights = edge_weights(colored_branch, tree_shape.tree_graph()).unwrap();
    assert_eq!(0, branch_weights[0].branch_choice);
    assert_eq!(3, branch_weights[1].branch_choice);
    assert_eq!(2, branch_weights[2].branch_choice);
    assert_eq!(4, branch_weights[3].branch_choice);
    assert_eq!(1, branch_weights[4].branch_choice);
    assert_eq!(4, branch_weights[5].branch_choice);
    assert_eq!(1, branch_weights[6].branch_choice);

    let tree_shape = TreeCanvas::builder().base(5).depth(6).build().draw_integer(&adic).unwrap();
    assert_eq!(5, tree_shape.base());
    assert_eq!(1, tree_shape.colored_branches().count());

    let colored_branch = tree_shape.colored_branches().next().unwrap();
    assert_eq!(7, colored_branch.0.len());
    let branch_weights = edge_weights(colored_branch, tree_shape.tree_graph()).unwrap();
    assert_eq!(0, branch_weights[0].branch_choice);
    assert_eq!(3, branch_weights[1].branch_choice);
    assert_eq!(2, branch_weights[2].branch_choice);
    assert_eq!(4, branch_weights[3].branch_choice);
    assert_eq!(1, branch_weights[4].branch_choice);
    assert_eq!(4, branch_weights[5].branch_choice);
    assert_eq!(1, branch_weights[6].branch_choice);

    let tree_shape = TreeCanvas::builder().base(5).depth(6).dangling_direction(None).build().draw_integer(&adic).unwrap();
    assert_eq!(5, tree_shape.base());
    assert_eq!(1, tree_shape.colored_branches().count());

    let colored_branch = &tree_shape.colored_branches().next().unwrap();
    assert_eq!(6, colored_branch.0.len());
    let branch_weights = edge_weights(colored_branch, tree_shape.tree_graph()).unwrap();
    assert_eq!(3, branch_weights[0].branch_choice);
    assert_eq!(2, branch_weights[1].branch_choice);
    assert_eq!(4, branch_weights[2].branch_choice);
    assert_eq!(1, branch_weights[3].branch_choice);
    assert_eq!(4, branch_weights[4].branch_choice);
    assert_eq!(1, branch_weights[5].branch_choice);

}

#[test]
fn correct_bounds() {

    let tree_shape = TreeCanvas::builder().base(5).depth(6).solid_full_tree().build().draw_full().unwrap();
    assert!(0. <= tree_shape.tree_graph().node_weights().map(|nw| nw.x).min_by(f64::total_cmp).unwrap());
    assert!(100. >= tree_shape.tree_graph().node_weights().map(|nw| nw.x).max_by(f64::total_cmp).unwrap());
    assert!(0. <= tree_shape.tree_graph().node_weights().map(|nw| nw.y).min_by(f64::total_cmp).unwrap());
    assert!(100. >= tree_shape.tree_graph().node_weights().map(|nw| nw.y).max_by(f64::total_cmp).unwrap());

    let adic = EAdic::new(5, vec![3, 2, 4, 1, 4, 1, 2]);
    let tree_shape = TreeCanvas::builder().base(5).depth(6).solid_full_tree().build().draw_integer(&adic).unwrap();
    assert!(0. <= tree_shape.tree_graph().node_weights().map(|nw| nw.x).min_by(f64::total_cmp).unwrap());
    assert!(100. >= tree_shape.tree_graph().node_weights().map(|nw| nw.x).max_by(f64::total_cmp).unwrap());
    assert!(0. <= tree_shape.tree_graph().node_weights().map(|nw| nw.y).min_by(f64::total_cmp).unwrap());
    assert!(100. >= tree_shape.tree_graph().node_weights().map(|nw| nw.y).max_by(f64::total_cmp).unwrap());

    let tree_shape = TreeCanvas::builder().base(5).depth(6).build().draw_integer(&adic).unwrap();
    assert!(0. <= tree_shape.tree_graph().node_weights().map(|nw| nw.x).min_by(f64::total_cmp).unwrap());
    assert!(100. >= tree_shape.tree_graph().node_weights().map(|nw| nw.x).max_by(f64::total_cmp).unwrap());
    assert!(0. <= tree_shape.tree_graph().node_weights().map(|nw| nw.y).min_by(f64::total_cmp).unwrap());
    assert!(100. >= tree_shape.tree_graph().node_weights().map(|nw| nw.y).max_by(f64::total_cmp).unwrap());

}

#[test]
fn graph_stats() {

    let tree_shape = TreeCanvas::builder().base(5).depth(4).solid_full_tree().build().draw_full().unwrap();
    let expected_num_branches = 625 + 125 + 25 + 5 + 1;
    assert_eq!(expected_num_branches + 1, tree_shape.tree_graph().node_count());
    assert_eq!(expected_num_branches, tree_shape.tree_graph().edge_count());
    assert_eq!(625 + 125 + 25 + 5 + 1 + 1, tree_shape.tree_graph().node_count());
    assert_eq!(625 + 125 + 25 + 5 + 1, tree_shape.tree_graph().edge_count());
    assert_eq!(1, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == -1).count());
    assert_eq!(1, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 0).count());
    assert_eq!(5, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 1).count());
    assert_eq!(25, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 2).count());
    assert_eq!(125, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 3).count());
    assert_eq!(625, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 4).count());
    assert_eq!((expected_num_branches-1)/5 + 1, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 0).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 1).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 2).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 3).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 4).count());

    let adic = EAdic::new(5, vec![3, 2, 4, 1, 4, 1, 2]);
    let tree_shape = TreeCanvas::builder().base(5).depth(4).solid_full_tree().build().draw_integer(&adic).unwrap();
    let expected_num_branches = 625 + 125 + 25 + 5 + 1;
    assert_eq!(expected_num_branches + 1, tree_shape.tree_graph().node_count());
    assert_eq!(expected_num_branches, tree_shape.tree_graph().edge_count());
    assert_eq!(1, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == -1).count());
    assert_eq!(1, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 0).count());
    assert_eq!(5, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 1).count());
    assert_eq!(25, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 2).count());
    assert_eq!(125, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 3).count());
    assert_eq!(625, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 4).count());
    assert_eq!((expected_num_branches-1)/5 + 1, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 0).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 1).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 2).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 3).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 4).count());

    let adic = EAdic::new(5, vec![3, 2, 4, 1, 4, 1, 2]);
    let tree_shape = TreeCanvas::builder().base(5).depth(4).build().draw_integer(&adic).unwrap();
    let expected_num_branches = 5 + 5 + 5 + 5 + 1;
    assert_eq!(expected_num_branches + 1, tree_shape.tree_graph().node_count());
    assert_eq!(expected_num_branches, tree_shape.tree_graph().edge_count());
    assert_eq!(1, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == -1).count());
    assert_eq!(1, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 0).count());
    assert_eq!(5, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 1).count());
    assert_eq!(5, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 2).count());
    assert_eq!(5, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 3).count());
    assert_eq!(5, tree_shape.tree_graph().node_weights().filter(|nw| nw.depth == 4).count());
    assert_eq!((expected_num_branches-1)/5 + 1, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 0).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 1).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 2).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 3).count());
    assert_eq!((expected_num_branches-1)/5, tree_shape.tree_graph().edge_weights().filter(|ew| ew.branch_choice == 4).count());

}

#[test]
fn edge_cases() {

    let adic = EAdic::new(11, vec![3, 2, 4, 10]);

    let tree_shape = TreeCanvas::builder().base(11).depth(4).build().draw_integer(&adic);
    assert!(tree_shape.is_ok());

    let tree_shape = TreeCanvas::builder().base(11).depth(4).twig_depth(4).build().draw_integer(&adic);
    assert_matches!(tree_shape, Err(AdicShapeError::TooLarge(_)));

}
