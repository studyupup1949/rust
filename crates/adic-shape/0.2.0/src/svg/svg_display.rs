use itertools::Itertools;
use svg::{Document, node::element as svg_el};
use crate::draw::{
    element::AdicEl,
    shape::DisplayShape,
    util::str_digits,
};


/// For plotting different SVGs
///
/// Use this trait when you have a shape and you want to print or save an SVG for it.
/// The [`create_svg_doc`](Self::create_svg_doc) method returns an [`svg::Document`] for the shape.
/// You can then for example call `svg::save(path, doc)` on the doc to save it to `path`.
pub trait SvgDisplay: DisplayShape {

    /// Create an SVG document for the shape
    fn create_svg_doc(&self) -> svg::Document {

        let viewbox_str = self.viewbox_str();
        let style_comps = self.shape_style_els();
        let svg_comps = self.shape_svg_els();

        // Wrap in svg
        let document = Document::new()
            .set("class", self.default_class())
            .set("viewBox", viewbox_str)
            .set("xmlns", "http://www.w3.org/2000/svg");
        style_comps.chain(svg_comps).fold(document, Document::add)

    }


    /// Iterator through the style elements for the svg
    fn shape_style_els(&self) -> impl Iterator<Item=svg_el::Element>;


    /// Iterator through all the components of the svg
    fn shape_svg_els(&self) -> impl Iterator<Item=svg_el::Element> {
        self.adic_els().map(|adic_el| match adic_el {
            AdicEl::Circle(c) => svg_el::Element::from({
                let mut circle = svg_el::Circle::new();
                if let Some(class) = c.class {
                    circle = circle.set("class", class);
                }
                circle = circle
                    .set("cx", str_digits(c.cx, 5))
                    .set("cy", str_digits(c.cy, 5))
                    .set("r", str_digits(c.r, 5));
                circle
            }),
            AdicEl::Path(p) => svg_el::Element::from({
                let mut path = svg_el::Path::new();
                if let Some(class) = p.class {
                    path = path.set("class", class);
                }
                path = path.set("d", p.d.into_iter().map(String::from).join(" "));
                path
            }),
            AdicEl::Text(t) => svg_el::Element::from({
                let mut text = svg_el::Text::new(t.content);
                if let Some(class) = t.class {
                    text = text.set("class", class);
                }
                if let Some(style) = t.style {
                    text = text.set("style", style);
                }
                text = text
                    .set("x", str_digits(t.x, 5))
                    .set("y", str_digits(t.y, 5))
                    .set("dx", str_digits(t.dx, 5))
                    .set("dy", str_digits(t.dy, 5));
                text
            })
        })
    }

}
