use adic::{traits::{AdicPrimitive, CanApproximate, HasDigits, PrimedFrom}, ZAdic};
use leptos::prelude::*;
use thaw::Flex;

use adic_shape::{
    leptos::{Collapse, ShapeCard},
    shape::{AdicCanvas, ClockCanvas, Direction, EuclideanCanvas, Orientation, TreeCanvas},
};


#[component]
pub fn ComparisonSuite() -> impl IntoView {

    // Comparison euclidean
    let adic_euclidean = |scaling: f64, depth: isize, num: &ZAdic| {
        let p = num.p();
        let num = num.approximation(depth.try_into().unwrap());
        move || EuclideanCanvas::builder()
            .characteristic_p_adic(p)
            .scaling(scaling).depth(depth)
            .draw_scaled_hulls()
            .direction(Direction::Up)
            .orientation(Orientation::CW)
            .build()
            .draw_integer(&num)
    };
    // Comparison clock
    let adic_clock = |depth: isize, num: &ZAdic| {
        let num = num.clone();
        move || ClockCanvas::builder()
            .base(num.base().into())
            .depth(depth)
            .build()
            .draw_integer(&num)
    };
    // Comparison tree
    let adic_tree = |depth: isize, num: &ZAdic| {
        let num = num.clone();
        move || TreeCanvas::builder()
            .base(num.base().into())
            .depth(depth)
            .build()
            .draw_integer(&num)
    };
    // Comparison tree
    let adic_full_tree = |depth: isize, num: &ZAdic| {
        let base = u32::from(num.base());
        let num = num.clone();
        move || TreeCanvas::builder()
            .base(base)
            .depth(depth)
            .solid_full_tree()
            .build()
            .draw_integer(&num)
    };

    let euclidean_clock_tree = view! {
        <Collapse title="Euclidean, clock, tree">

            <Flex>

                <ShapeCard class="euclidean"
                    title="Euclidean 3-adic 1"
                    shape=adic_euclidean(2.2, 5, &ZAdic::primed_from(3, 1))
                />

                <ShapeCard class="clock"
                    title="Clock 3-adic 1"
                    shape=adic_clock(5, &ZAdic::primed_from(3, 1))
                />

                <ShapeCard class="tree"
                    title="Tree 3-adic 1"
                    shape=adic_tree(5, &ZAdic::primed_from(3, 1))
                />

                <ShapeCard class="tree"
                    title="Full tree 3-adic 1"
                    shape=adic_full_tree(5, &ZAdic::primed_from(3, 1))
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="Euclidean 3-adic -1"
                    shape=adic_euclidean(2.2, 5, &ZAdic::primed_from(3, -1))
                />

                <ShapeCard class="clock"
                    title="Clock 3-adic -1"
                    shape=adic_clock(5, &ZAdic::primed_from(3, -1))
                />

                <ShapeCard class="tree"
                    title="Tree 3-adic -1"
                    shape=adic_tree(5, &ZAdic::primed_from(3, -1))
                />

                <ShapeCard class="tree"
                    title="Full tree 3-adic -1"
                    shape=adic_full_tree(5, &ZAdic::primed_from(3, -1))
                />

            </Flex>

        </Collapse>
    };

    let compare_view = move |idx: usize| {
        let roots_of_unity_5 = ZAdic::roots_of_unity(5, 4).unwrap().into_roots().collect::<Vec<_>>();
        let root = &roots_of_unity_5[idx];
        view! {
            <ShapeCard class="euclidean"
                title=format!("Euclidean 5-adic root of unity {}", idx+1)
                shape=adic_euclidean(2.6, 4, root)
            />

            <ShapeCard class="clock"
                title=format!("Clock 5-adic root of unity {}", idx+1)
                shape=adic_clock(4, root)
            />

            <ShapeCard class="tree"
                title=format!("Tree 5-adic root of unity {}", idx+1)
                shape=adic_tree(4, root)
            />
        }
    };

    let root_5_comparison = view! {
        <Collapse title="5-adic roots of unity">

            <Flex>
                { compare_view(0) }
            </Flex>

            <Flex>
                { compare_view(1) }
            </Flex>

            <Flex>
                { compare_view(2) }
            </Flex>

            <Flex>
                { compare_view(3) }
            </Flex>

        </Collapse>
    };

    view! {
        <section class="boxed-section">
            {euclidean_clock_tree}
            {root_5_comparison}
        </section>
    }

}
