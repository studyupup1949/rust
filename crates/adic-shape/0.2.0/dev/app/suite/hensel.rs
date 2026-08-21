use itertools::{Itertools, join};
use leptos::prelude::*;
use thaw::Flex;

use adic_shape::adic::{
    traits::CanTruncate,
    EAdic, Polynomial, Variety, ZAdic,
};
use adic_shape::{
    animation::{Frame, FrameReel},
    error::AdicShapeError,
    leptos::{AnimatedShapeCard, Collapse},
    shape::{AdicCanvas, TreeCanvas, TreeShape},
};


#[component]
pub fn HenselSuite() -> impl IntoView {

    let max_num_digits = 10;
    let imax_num_digits = 10;

    // Hensel (x+23)(x-3)(x-2) = x^3 + 18 x^2 - 109 x + 138
    let polynomial = Polynomial::<EAdic>::new_with_prime(5, vec![138, -109, 18, 1]);
    let variety = polynomial.variety(imax_num_digits).unwrap();
    let variety = Variety::new(
        variety.into_roots().map(|q| q.try_into_integer().unwrap()).collect::<Vec<_>>()
    );
    let animated_hensel_reel = animated_reel(&variety, max_num_digits);

    // Hensel 5-adic roots of unity
    let variety = ZAdic::roots_of_unity(5, max_num_digits).unwrap();
    let roots_of_unity_reel = animated_reel(&variety, max_num_digits);

    let animated_hensel = view! {
        <Collapse title="Animated Hensel">

            <Flex>

                <AnimatedShapeCard class="tree"
                    title="Hensel lift (x+23)(x-3)(x-2)"
                    shape_reel=animated_hensel_reel
                />

                <AnimatedShapeCard class="tree"
                    title="Hensel lift roots of unity"
                    shape_reel=roots_of_unity_reel
                />

            </Flex>

        </Collapse>
    };

    view! {
        <section class="boxed-section">
            <h3>"Hensel examples"</h3>
            {animated_hensel}
        </section>
    }

}


fn animated_reel(
    variety: &Variety<ZAdic>,
    max_num_digits: usize,
) -> FrameReel<Result<TreeShape, AdicShapeError>> {

    FrameReel::new(
        (0..=max_num_digits).map(|num| {
            let canvas = TreeCanvas::builder()
                .base(variety.p().unwrap().into())
                .depth(num.try_into().unwrap())
                .build();
            let shape_triple_root = canvas.draw_integers(variety.roots());
            let label = [
                "( ".to_string(),
                join(variety.roots().map(|r| r.truncation(num)).unique(), ",   "),
                " )".to_string(),
            ].concat();
            Frame::from((u32::try_from(num).unwrap(), shape_triple_root, label))
        }).collect::<Vec<_>>(),
        u32::try_from(max_num_digits+1).unwrap()
    )

}
