use svg::node::element as svg_el;
use crate::ClockShape;
use super::SvgDocDisplay;


/// SVG for an adic clock
///
/// ```
/// # use adic::radic;
/// # use adic_shape::{ClockShape, ClockShapeOptions, SvgDocDisplay};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let neg_one_fourth = radic!(5, [], [1]);
/// let num_digits = 6;
/// let adic_clock = ClockShape::new(&neg_one_fourth, num_digits, ClockShapeOptions::default())?;
/// let clock_svg = adic_clock.create_svg_doc();
/// # Ok(()) }
/// ```
impl SvgDocDisplay for ClockShape {

    fn shape_style_els(
        &self,
    ) -> impl Iterator<Item=svg_el::Element> {
        clock_style_instructions()
    }

}


fn clock_style_instructions() -> impl Iterator<Item=svg_el::Element> {
    let style_el = svg_el::Style::new("
svg {
    background: var(--clock-svg-background, white);
}
svg .clock-circle {
    fill: transparent;
    stroke: var(--clock-svg-circle-color, black);
    stroke-width: 0.2;
}
svg .clock-sub-circle {
    fill: transparent;
    stroke: var(--clock-svg-sub-circle-color, black);
    stroke-width: 0.2;
    stroke-dasharray: 1, 4;
}
svg .clock-hand-path {
    fill: transparent;
    stroke: var(--clock-svg-hand-color, black);
    stroke-width: 0.2;
}
svg .clock-head-circle {
    fill: var(--clock-svg-hand-color, black);
    stroke: var(--clock-svg-hand-color, black);
    stroke-width: 0.2;
}
svg .tick-label {
    fill: var(--clock-svg-tick-color, black);
    text-anchor: middle;
    dominant-baseline: middle;
}
"
    );
    std::iter::once(svg_el::Element::from(style_el))
}



#[cfg(test)]
mod test {

    use adic::zadic_approx;
    use crate::{ClockMovement, ClockShape, ClockShapeOptions, SvgDocDisplay};

    #[test]
    fn basic_clock() {

        let adic_data = zadic_approx!(5, 5, [0, 1, 2, 3, 4]);
        let shape_options = ClockShapeOptions {
            clock_movement: ClockMovement::Ticking,
            show_hand_radii: true,
            viewbox_width: 100,
            viewbox_height: 100,
        };

        // Create the clock
        let clock_shape = ClockShape::new(&adic_data, 5, shape_options).unwrap();

        let clock = clock_shape.create_svg_doc();

        let expected = [
            "<svg class=\"adic-clock\" viewBox=\"0 0 100 100\" xmlns=\"http://www.w3.org/2000/svg\">",
               "<style>",
                    "svg {",
                    "    background: var(--clock-svg-background, white);",
                    "}",
                    "svg .clock-circle {",
                    "    fill: transparent;",
                    "    stroke: var(--clock-svg-circle-color, black);",
                    "    stroke-width: 0.2;",
                    "}",
                    "svg .clock-sub-circle {",
                    "    fill: transparent;",
                    "    stroke: var(--clock-svg-sub-circle-color, black);",
                    "    stroke-width: 0.2;",
                    "    stroke-dasharray: 1, 4;",
                    "}",
                    "svg .clock-hand-path {",
                    "    fill: transparent;",
                    "    stroke: var(--clock-svg-hand-color, black);",
                    "    stroke-width: 0.2;",
                    "}",
                    "svg .clock-head-circle {",
                    "    fill: var(--clock-svg-hand-color, black);",
                    "    stroke: var(--clock-svg-hand-color, black);",
                    "    stroke-width: 0.2;",
                    "}",
                    "svg .tick-label {",
                    "    fill: var(--clock-svg-tick-color, black);",
                    "    text-anchor: middle;",
                    "    dominant-baseline: middle;",
                    "}",
                "</style>",
                "<circle class=\"clock-circle\" cx=\"50\" cy=\"50\" r=\"40\"/>",
                "<circle class=\"clock-sub-circle\" cx=\"50\" cy=\"50\" r=\"6.666666666666666\"/>",
                "<circle class=\"clock-sub-circle\" cx=\"50\" cy=\"50\" r=\"13.333333333333332\"/>",
                "<circle class=\"clock-sub-circle\" cx=\"50\" cy=\"50\" r=\"20\"/>",
                "<circle class=\"clock-sub-circle\" cx=\"50\" cy=\"50\" r=\"26.666666666666664\"/>",
                "<circle class=\"clock-sub-circle\" cx=\"50\" cy=\"50\" r=\"33.333333333333336\"/>",
                "<path class=\"clock-hand-path\" d=\"M 50 50 L 50 43.333333333333336\"/>",
                "<circle class=\"clock-head-circle\" cx=\"50\" cy=\"43.333333333333336\" r=\"0.5\"/>",
                "<path class=\"clock-hand-path\" d=\"M 50 50 L 62.68075355060205 45.87977340833403\"/>",
                "<circle class=\"clock-head-circle\" cx=\"62.68075355060205\" cy=\"45.87977340833403\" r=\"0.5\"/>",
                "<path class=\"clock-hand-path\" d=\"M 50 50 L 61.75570504584947 66.18033988749895\"/>",
                "<circle class=\"clock-head-circle\" cx=\"61.75570504584947\" cy=\"66.18033988749895\" r=\"0.5\"/>",
                "<path class=\"clock-hand-path\" d=\"M 50 50 L 34.32572660553406 71.57378651666527\"/>",
                "<circle class=\"clock-head-circle\" cx=\"34.32572660553406\" cy=\"71.57378651666527\" r=\"0.5\"/>",
                "<path class=\"clock-hand-path\" d=\"M 50 50 L 18.298116123494875 39.69943352083509\"/>",
                "<circle class=\"clock-head-circle\" cx=\"18.298116123494875\" cy=\"39.69943352083509\" r=\"0.5\"/>",
                "<path class=\"clock-hand-path\" d=\"M 50 12 L 50 10\"/>",
                "<path class=\"clock-hand-path\" d=\"M 86.14014761921584 38.257354213751995 L 88.04226065180615 37.6393202250021\"/>",
                "<path class=\"clock-hand-path\" d=\"M 72.33583958711398 80.74264578624799 L 73.51141009169893 82.36067977499789\"/>",
                "<path class=\"clock-hand-path\" d=\"M 27.664160412886027 80.742645786248 L 26.48858990830108 82.3606797749979\"/>",
                "<path class=\"clock-hand-path\" d=\"M 13.859852380784162 38.257354213752 L 11.957739348193854 37.63932022500211\"/>",
                "<text class=\"tick-label\" dx=\"0\" dy=\"-4\" style=\"position: fixed; font-size: 4pt;\" x=\"50\" y=\"10\">0</text>",
                "<text class=\"tick-label\" dx=\"3.8042260651806146\" dy=\"-1.2360679774997898\" style=\"position: fixed; font-size: 4pt;\" x=\"88.04226065180615\" y=\"37.6393202250021\">1</text>",
                "<text class=\"tick-label\" dx=\"2.3511410091698934\" dy=\"3.2360679774997894\" style=\"position: fixed; font-size: 4pt;\" x=\"73.51141009169893\" y=\"82.36067977499789\">2</text>",
                "<text class=\"tick-label\" dx=\"-2.351141009169892\" dy=\"3.2360679774997907\" style=\"position: fixed; font-size: 4pt;\" x=\"26.48858990830108\" y=\"82.3606797749979\">3</text>",
                "<text class=\"tick-label\" dx=\"-3.8042260651806146\" dy=\"-1.2360679774997891\" style=\"position: fixed; font-size: 4pt;\" x=\"11.957739348193854\" y=\"37.63932022500211\">4</text>",
            "</svg>",
        ].join("\n");

        for (e, c) in expected.split('\n').zip(clock.to_string().split('\n')) {
            assert_eq!(e, c);
        }
        assert_eq!(expected, clock.to_string());

    }

}
