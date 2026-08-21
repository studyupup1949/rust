use svg::node::element as svg_el;
use crate::shape::ClockShape;
use super::SvgDisplay;


/// SVG for an adic clock
///
/// ```
/// # use adic::EAdic;
/// # use adic_shape::{shape::{AdicCanvas, ClockCanvas}, svg::SvgDisplay};
/// let canvas = ClockCanvas::builder().base(5).depth(6).build();
/// let neg_one_fourth = EAdic::new_repeating(5, vec![], vec![1]);
/// let clock_shape = canvas.draw_integer(&neg_one_fourth)?;
/// let clock_svg = clock_shape.create_svg_doc();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
impl SvgDisplay for ClockShape {

    fn shape_style_els(
        &self,
    ) -> impl Iterator<Item=svg_el::Element> {
        clock_style_instructions()
    }

}


fn clock_style_instructions() -> impl Iterator<Item=svg_el::Element> {
    let style_el = svg_el::Style::new("
svg {
    background: white;
}
svg .clock-circle {
    fill: transparent;
    stroke: black;
    stroke-width: 0.2;
}
svg .clock-zero-val-circle {
    fill: transparent;
    stroke: red;
    stroke-width: 0.2;
}
svg .clock-val-circle {
    fill: transparent;
    stroke: black;
    stroke-width: 0.2;
    stroke-dasharray: 1, 4;
}
svg .clock-hand-path {
    fill: transparent;
    stroke: black;
    stroke-width: 0.2;
}
svg .clock-head-circle {
    fill: black;
    stroke: black;
    stroke-width: 0.2;
}
svg .tick-label {
    fill: black;
    text-anchor: middle;
    dominant-baseline: middle;
}
"
    );
    std::iter::once(svg_el::Element::from(style_el))
}



#[cfg(test)]
mod test {

    use adic::ZAdic;
    use crate::{
        shape::{AdicCanvas, ClockCanvas, ClockMovement},
        svg::SvgDisplay,
    };

    #[test]
    fn basic_clock() {

        // Create the clock
        let num_digits = 5;
        let adic_data = ZAdic::new_approx(5, num_digits, vec![0, 1, 2, 3, 4]);
        let clock_canvas = ClockCanvas::builder()
            .base(5)
            .depth(num_digits.try_into().unwrap())
            .clock_movement(ClockMovement::Ticking)
            .show_val_circles(true)
            .build();
        let clock_shape = clock_canvas.draw_integer(&adic_data).unwrap();

        let clock = clock_shape.create_svg_doc();

        let expected = r#"<svg class="adic-clock" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
<style>
svg {
    background: white;
}
svg .clock-circle {
    fill: transparent;
    stroke: black;
    stroke-width: 0.2;
}
svg .clock-zero-val-circle {
    fill: transparent;
    stroke: red;
    stroke-width: 0.2;
}
svg .clock-val-circle {
    fill: transparent;
    stroke: black;
    stroke-width: 0.2;
    stroke-dasharray: 1, 4;
}
svg .clock-hand-path {
    fill: transparent;
    stroke: black;
    stroke-width: 0.2;
}
svg .clock-head-circle {
    fill: black;
    stroke: black;
    stroke-width: 0.2;
}
svg .tick-label {
    fill: black;
    text-anchor: middle;
    dominant-baseline: middle;
}
</style>
<circle class="clock-circle" cx="50" cy="50" r="40"/>
<circle class="clock-val-circle" cx="50" cy="50" r="6.66667"/>
<circle class="clock-val-circle" cx="50" cy="50" r="13.33333"/>
<circle class="clock-val-circle" cx="50" cy="50" r="20"/>
<circle class="clock-val-circle" cx="50" cy="50" r="26.66667"/>
<circle class="clock-val-circle" cx="50" cy="50" r="33.33333"/>
<path class="clock-hand-path" d="M 50 50 L 50 43.33333"/>
<circle class="clock-head-circle" cx="50" cy="43.33333" r="0.5"/>
<path class="clock-hand-path" d="M 50 50 L 62.68075 45.87977"/>
<circle class="clock-head-circle" cx="62.68075" cy="45.87977" r="0.5"/>
<path class="clock-hand-path" d="M 50 50 L 61.75571 66.18034"/>
<circle class="clock-head-circle" cx="61.75571" cy="66.18034" r="0.5"/>
<path class="clock-hand-path" d="M 50 50 L 34.32573 71.57379"/>
<circle class="clock-head-circle" cx="34.32573" cy="71.57379" r="0.5"/>
<path class="clock-hand-path" d="M 50 50 L 18.29812 39.69943"/>
<circle class="clock-head-circle" cx="18.29812" cy="39.69943" r="0.5"/>
<path class="clock-hand-path" d="M 50 12 L 50 10"/>
<path class="clock-hand-path" d="M 86.14015 38.25735 L 88.04226 37.63932"/>
<path class="clock-hand-path" d="M 72.33584 80.74265 L 73.51141 82.36068"/>
<path class="clock-hand-path" d="M 27.66416 80.74265 L 26.48859 82.36068"/>
<path class="clock-hand-path" d="M 13.85985 38.25735 L 11.95774 37.63932"/>
<text class="tick-label" dx="0" dy="-4" style="position: fixed; font-size: 4pt;" x="50" y="10">0</text>
<text class="tick-label" dx="3.80423" dy="-1.23607" style="position: fixed; font-size: 4pt;" x="88.04226" y="37.63932">1</text>
<text class="tick-label" dx="2.35114" dy="3.23607" style="position: fixed; font-size: 4pt;" x="73.51141" y="82.36068">2</text>
<text class="tick-label" dx="-2.35114" dy="3.23607" style="position: fixed; font-size: 4pt;" x="26.48859" y="82.36068">3</text>
<text class="tick-label" dx="-3.80423" dy="-1.23607" style="position: fixed; font-size: 4pt;" x="11.95774" y="37.63932">4</text>
</svg>"#;

        for (e, c) in expected.split('\n').zip(clock.to_string().split('\n')) {
            assert_eq!(e, c);
        }
        assert_eq!(expected, clock.to_string());

    }

}
