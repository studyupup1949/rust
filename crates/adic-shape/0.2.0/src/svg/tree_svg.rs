use svg::node::element as svg_el;

use crate::shape::TreeShape;
use super::SvgDisplay;
// use super::tree_labeller::TreeLabeller;


/// SVG for an adic tree
///
/// ```
/// # use adic::EAdic;
/// # use adic_shape::{shape::{AdicCanvas, TreeCanvas}, svg::SvgDisplay};
/// let neg_one_fourth = EAdic::new_repeating(5, vec![], vec![1]);
/// let num_digits = 6;
/// let canvas = TreeCanvas::builder()
///     .base(5).depth(num_digits).solid_full_tree().build();
/// let tree_shape = canvas.draw_full()?;
/// let tree_svg = tree_shape.create_svg_doc();
/// let canvas = TreeCanvas::builder()
///     .base(5).depth(num_digits).build();
/// let tree_shape = canvas.draw_integer(&neg_one_fourth)?;
/// let tree_svg = tree_shape.create_svg_doc();
///
/// let neg_one_fourth = EAdic::new_repeating(5, vec![], vec![1]);
/// let pos_one_fourth = -neg_one_fourth.clone();
/// let adics = vec![neg_one_fourth, pos_one_fourth];
/// let canvas = TreeCanvas::builder()
///     .base(5).depth(num_digits).dashed_full_tree().build();
/// let tree_shape = canvas.draw_integers(&adics)?;
/// let tree_svg = tree_shape.create_svg_doc();
/// let canvas = TreeCanvas::builder()
///     .base(5).depth(num_digits).build();
/// let tree_shape = canvas.draw_integers(&adics)?;
/// let tree_svg = tree_shape.create_svg_doc();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
impl SvgDisplay for TreeShape {

    fn shape_style_els(
        &self,
    ) -> impl Iterator<Item=svg_el::Element> {
        tree_style_instructions()
    }

}



fn tree_style_instructions() -> impl Iterator<Item=svg_el::Element> {
    let style_el = svg_el::Style::new("
svg {
    background: white;
}
svg .tree-path {
    fill: transparent;
    stroke-width: 0.3;
}
svg .tree-path-default {
    stroke: black;
}
svg .tree-path-combined {
    stroke: #2040C0;
}
svg .tree-path-color-0 {
    stroke: #FF9000;
}
svg .tree-path-color-1 {
    stroke: #00FF90;
}
svg .tree-path-color-2 {
    stroke: #9000FF;
}
svg .tree-path-color-3 {
    stroke: #FF9090;
}
svg .tree-path-color-4 {
    stroke: #90FF90;
}
svg .tree-path-color-5 {
    stroke: #9090FF;
}
svg .tree-path-solid {
    stroke-dasharray: none;
}
svg .tree-path-dashed {
    stroke-dasharray: 1, 4;
}
svg .tree-zero-val-level {
    fill: transparent;
    stroke: red;
    stroke-width: 0.2;
}
svg .tree-val-level {
    fill: transparent;
    stroke-width: 0.2;
    stroke-dasharray: 1, 1;
    stroke: black;
}
"
    );
    std::iter::once(svg_el::Element::from(style_el))
}



#[cfg(test)]
mod test {

    use adic::ZAdic;
    use crate::{
        shape::{AdicCanvas, TreeCanvas},
        svg::SvgDisplay,
    };

    #[test]
    fn basic_tree() {

        // Create the tree
        let p = 5;
        let num_digits = 2;
        let adic_data = ZAdic::new_approx(p, num_digits, vec![1, 3]);
        let tree_canvas = TreeCanvas::builder()
            .base(p)
            .depth(num_digits.try_into().unwrap())
            .solid_full_tree()
            .build();
        let tree_shape = tree_canvas.draw_integer(&adic_data).unwrap();

        let tree = tree_shape.create_svg_doc();

        let expected = r#"<svg class="adic-tree" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
<style>
svg {
    background: white;
}
svg .tree-path {
    fill: transparent;
    stroke-width: 0.3;
}
svg .tree-path-default {
    stroke: black;
}
svg .tree-path-combined {
    stroke: #2040C0;
}
svg .tree-path-color-0 {
    stroke: #FF9000;
}
svg .tree-path-color-1 {
    stroke: #00FF90;
}
svg .tree-path-color-2 {
    stroke: #9000FF;
}
svg .tree-path-color-3 {
    stroke: #FF9090;
}
svg .tree-path-color-4 {
    stroke: #90FF90;
}
svg .tree-path-color-5 {
    stroke: #9090FF;
}
svg .tree-path-solid {
    stroke-dasharray: none;
}
svg .tree-path-dashed {
    stroke-dasharray: 1, 4;
}
svg .tree-zero-val-level {
    fill: transparent;
    stroke: red;
    stroke-width: 0.2;
}
svg .tree-val-level {
    fill: transparent;
    stroke-width: 0.2;
    stroke-dasharray: 1, 1;
    stroke: black;
}
</style>
<path class="tree-path tree-path-default tree-path-solid" d="M 50 80 L 10 40 M 10 40 L 2 0 M 10 40 L 6 0 M 10 40 L 10 0 M 10 40 L 14 0 M 10 40 L 18 0 M 30 40 L 22 0 M 30 40 L 26 0 M 30 40 L 30 0 M 30 40 L 38 0 M 50 80 L 50 40 M 50 40 L 42 0 M 50 40 L 46 0 M 50 40 L 50 0 M 50 40 L 54 0 M 50 40 L 58 0 M 50 80 L 70 40 M 70 40 L 62 0 M 70 40 L 66 0 M 70 40 L 70 0 M 70 40 L 74 0 M 70 40 L 78 0 M 50 80 L 90 40 M 90 40 L 82 0 M 90 40 L 86 0 M 90 40 L 90 0 M 90 40 L 94 0 M 90 40 L 98 0"/>
<path class="tree-path tree-path-color-0 tree-path-solid" d="M 100 100 L 50 80 M 50 80 L 30 40 M 30 40 L 34 0"/>
</svg>"#;

        for (e, t) in expected.split('\n').zip(tree.to_string().split('\n')) {
            assert_eq!(e, t);
        }

        assert_eq!(expected, tree.to_string());

    }

}
