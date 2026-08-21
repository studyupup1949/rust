use svg::node::element as svg_el;
use crate::shape::EuclideanShape;
use super::SvgDisplay;


/// SVG for an adic euclidean
///
/// ```
/// # use adic::EAdic;
/// # use adic_shape::{shape::{AdicCanvas, EuclideanCanvas}, svg::SvgDisplay};
/// let prime = 5;
/// let scaling = 1.5;
/// let depth = 10;
/// let canvas = EuclideanCanvas::builder()
///     .characteristic_p_adic(prime)
///     .depth(depth)
///     .scaling(scaling)
///     .build();
/// let neg_one_fourth = EAdic::new_repeating(5, vec![], vec![1]);
/// let euclidean_shape = canvas.draw_integer(&neg_one_fourth)?;
/// let euclidean_svg = euclidean_shape.create_svg_doc();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
impl SvgDisplay for EuclideanShape {

    fn shape_style_els(
        &self,
    ) -> impl Iterator<Item=svg_el::Element> {
        euclidean_style_instructions()
    }

}


fn euclidean_style_instructions() -> impl Iterator<Item=svg_el::Element> {
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
svg .euclidean-convex-hull {
    fill: black;
}
svg .euclidean-dot {
    fill: black;
}
svg .euclidean-enclosing-disk {
    fill: transparent;
    stroke: black;
    stroke-width: 0.1;
    stroke-dasharray: 0.3, 2;
}
svg .euclidean-zero-val-hull {
    fill: transparent;
    stroke: red;
    stroke-width: 0.2;
}
svg .euclidean-val-hull {
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

    use crate::{
        shape::{AdicCanvas, EuclideanCanvas},
        svg::SvgDisplay,
    };

    #[test]
    fn basic_euclidean() {

        // Create the euclidean
        let sierpinski_vec_digits = vec![(0.0, 0.0), (1.0, 0.0), (0.5, 1.0)];
        let scaling = 3.0;
        let depth = 2;
        let canvas = EuclideanCanvas::builder()
            .fixed_hulls(sierpinski_vec_digits)
            .scaling(scaling).depth(depth)
            .solid_full_tree()
            .draw_scaled_hulls()
            .build();
        let shape = canvas.draw_full().unwrap();

        let euclidean = shape.create_svg_doc();

        let expected = r#"<svg class="adic-euclidean" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
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
svg .euclidean-convex-hull {
    fill: black;
}
svg .euclidean-dot {
    fill: black;
}
svg .euclidean-enclosing-disk {
    fill: transparent;
    stroke: black;
    stroke-width: 0.1;
    stroke-dasharray: 0.3, 2;
}
svg .euclidean-zero-val-hull {
    fill: transparent;
    stroke: red;
    stroke-width: 0.2;
}
svg .euclidean-val-hull {
    fill: transparent;
    stroke-width: 0.2;
    stroke-dasharray: 1, 1;
    stroke: black;
}
</style>
<path class="euclidean-convex-hull" d="M 0.5 99.5 L 11.5 99.5 L 6 88.5 L 0.5 99.5 M 22.5 99.5 L 33.5 99.5 L 28 88.5 L 22.5 99.5 M 11.5 77.5 L 22.5 77.5 L 17 66.5 L 11.5 77.5 M 66.5 99.5 L 77.5 99.5 L 72 88.5 L 66.5 99.5 M 88.5 99.5 L 99.5 99.5 L 94 88.5 L 88.5 99.5 M 77.5 77.5 L 88.5 77.5 L 83 66.5 L 77.5 77.5 M 33.5 33.5 L 44.5 33.5 L 39 22.5 L 33.5 33.5 M 55.5 33.5 L 66.5 33.5 L 61 22.5 L 55.5 33.5 M 44.5 11.5 L 55.5 11.5 L 50 0.5 L 44.5 11.5"/>
<path class="tree-path tree-path-default tree-path-solid" d="M 2.33333 97.66667 L 2.33333 97.66667 M 2.33333 97.66667 L 68.33333 97.66667 M 2.33333 97.66667 L 35.33333 31.66667 M 2.33333 97.66667 L 2.33333 97.66667 M 2.33333 97.66667 L 24.33333 97.66667 M 2.33333 97.66667 L 13.33333 75.66667 M 68.33333 97.66667 L 68.33333 97.66667 M 68.33333 97.66667 L 90.33333 97.66667 M 68.33333 97.66667 L 79.33333 75.66667 M 35.33333 31.66667 L 35.33333 31.66667 M 35.33333 31.66667 L 57.33333 31.66667 M 35.33333 31.66667 L 46.33333 9.66667 M 2.33333 97.66667 L 2.33333 97.66667 M 2.33333 97.66667 L 9.66667 97.66667 M 2.33333 97.66667 L 6 90.33333 M 24.33333 97.66667 L 24.33333 97.66667 M 24.33333 97.66667 L 31.66667 97.66667 M 24.33333 97.66667 L 28 90.33333 M 13.33333 75.66667 L 13.33333 75.66667 M 13.33333 75.66667 L 20.66667 75.66667 M 13.33333 75.66667 L 17 68.33333 M 68.33333 97.66667 L 68.33333 97.66667 M 68.33333 97.66667 L 75.66667 97.66667 M 68.33333 97.66667 L 72 90.33333 M 90.33333 97.66667 L 90.33333 97.66667 M 90.33333 97.66667 L 97.66667 97.66667 M 90.33333 97.66667 L 94 90.33333 M 79.33333 75.66667 L 79.33333 75.66667 M 79.33333 75.66667 L 86.66667 75.66667 M 79.33333 75.66667 L 83 68.33333 M 35.33333 31.66667 L 35.33333 31.66667 M 35.33333 31.66667 L 42.66667 31.66667 M 35.33333 31.66667 L 39 24.33333 M 57.33333 31.66667 L 57.33333 31.66667 M 57.33333 31.66667 L 64.66667 31.66667 M 57.33333 31.66667 L 61 24.33333 M 46.33333 9.66667 L 46.33333 9.66667 M 46.33333 9.66667 L 53.66667 9.66667 M 46.33333 9.66667 L 50 2.33333"/>
</svg>"#;

        for (e, c) in expected.split('\n').zip(euclidean.to_string().split('\n')) {
            assert_eq!(e, c);
        }
        assert_eq!(expected, euclidean.to_string());

    }

}
