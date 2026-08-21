use svg::node::element as svg_el;

use crate::TreeShape;
use super::SvgDocDisplay;
// use super::tree_labeller::TreeLabeller;


/// SVG for an adic tree
///
/// ```
/// # use adic::radic;
/// # use adic_shape::{TreeShape, TreeShapeOptions, SvgDocDisplay};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let neg_one_fourth = radic!(5, [], [1]);
/// let num_digits = 6;
/// let adic_tree = TreeShape::full_tree(5, num_digits, TreeShapeOptions::default())?;
/// let tree_svg = adic_tree.create_svg_doc();
/// let adic_tree = TreeShape::adic_number_full_tree(&neg_one_fourth, num_digits, TreeShapeOptions::default())?;
/// let tree_svg = adic_tree.create_svg_doc();
/// let adic_tree = TreeShape::zoomed_tree(&neg_one_fourth, num_digits, TreeShapeOptions::default())?;
/// let tree_svg = adic_tree.create_svg_doc();
/// # Ok(()) }
/// ```
impl SvgDocDisplay for TreeShape {

    fn shape_style_els(
        &self,
    ) -> impl Iterator<Item=svg_el::Element> {
        tree_style_instructions()
    }

}



fn tree_style_instructions() -> impl Iterator<Item=svg_el::Element> {
    let style_el = svg_el::Style::new("
svg {
    background: var(--tree-svg-background, white);
}
svg .tree-path {
    fill: transparent;
    stroke: var(--tree-path, black);
    stroke-width: 0.3;
}
svg .gold-path {
    fill: transparent;
    stroke: var(--gold-path, darkorange);
    stroke-width: 0.5;
}
"
    );
    std::iter::once(svg_el::Element::from(style_el))
}



#[cfg(test)]
mod test {

    use adic::zadic_approx;
    use crate::{TreeShape, TreeShapeOptions, SvgDocDisplay};

    #[test]
    fn basic_tree() {

        let adic_data = zadic_approx!(5, 2, [1, 3]);
        let num_digits = 2;
        let shape_options = TreeShapeOptions::default();

        // Create the tree
        let tree_shape = TreeShape::adic_number_full_tree(&adic_data, num_digits, shape_options).unwrap();

        let tree = tree_shape.create_svg_doc();

        let expected = [
            "<svg class=\"adic-tree\" viewBox=\"0 0 100 100\" xmlns=\"http://www.w3.org/2000/svg\">",
                "<style>",
                      "svg {",
                      "    background: var(--tree-svg-background, white);",
                      "}",
                      "svg .tree-path {",
                      "    fill: transparent;",
                      "    stroke: var(--tree-path, black);",
                      "    stroke-width: 0.3;",
                      "}",
                      "svg .gold-path {",
                      "    fill: transparent;",
                      "    stroke: var(--gold-path, darkorange);",
                      "    stroke-width: 0.5;",
                      "}",
                  "</style>",
                  &[
                      "<path class=\"tree-path\" d=\"",
                          "M 100 100 L 50 80 M 50 80 L 9.999999999999998 40 M 9.999999999999998 40 L 1.9999999999999962 0 M 9.999999999999998 40 L 5.999999999999997 0 ",
                          "M 9.999999999999998 40 L 9.999999999999998 0 M 9.999999999999998 40 L 13.999999999999996 0 M 9.999999999999998 40 L 18 0 M 50 80 L 30 40 ",
                          "M 30 40 L 21.999999999999996 0 M 30 40 L 26 0 M 30 40 L 30 0 M 30 40 L 34 0 M 30 40 L 38 0 M 50 80 L 50 40 M 50 40 L 42 0 M 50 40 L 46 0 ",
                          "M 50 40 L 50 0 M 50 40 L 54 0 M 50 40 L 58.00000000000001 0 M 50 80 L 70 40 M 70 40 L 61.999999999999986 0 M 70 40 L 65.99999999999999 0 ",
                          "M 70 40 L 70 0 M 70 40 L 74 0 M 70 40 L 78 0 M 50 80 L 90 40 M 90 40 L 82 0 M 90 40 L 86 0 M 90 40 L 90 0 M 90 40 L 94 0 M 90 40 L 98 0",
                      "\"/>",
                  ].join(""),
                  "<path class=\"gold-path\" d=\"M 100 100 L 50 80 M 50 80 L 30 40 M 30 40 L 34 0\"/>",
            "</svg>",
        ].join("\n");

        for (e, t) in expected.split('\n').zip(tree.to_string().split('\n')) {
            assert_eq!(e, t);
        }

        assert_eq!(expected, tree.to_string());

    }

}
