use std::f64::consts::TAU;
use adic::{
    traits::{AdicInteger, AdicPrimitive, PrimedFrom},
    EAdic, Polynomial, QAdic, Variety, ZAdic,
};
use leptos::prelude::*;
use num::{ToPrimitive, Rational32};
use thaw::{Flex, Slider, SliderLabel, SpinButton};

use adic_shape::{
    leptos::{Collapse, ShapeCard},
    shape::{AdicCanvas, Direction, EuclideanCanvas, Orientation},
};


#[component]
pub fn EuclideanSuite() -> impl IntoView {

    let sqrt2_over_2 = 2.0_f64.sqrt() * 0.5;
    let sqrt3_over_6 = 3.0_f64.sqrt() / 6.0;

    // Sierpinsky
    let sierpinsky = move || EuclideanCanvas::builder()
        .fixed_hulls(vec![(0.0, 0.0), (1.0, 0.0), (0.5, sqrt2_over_2)])
        .scaling(2.1).depth(5)
        .draw_scaled_hulls()
        .build()
        .draw_full();

    // Rectangular with a dashed full tree
    let rectangle = move || EuclideanCanvas::builder()
        .fixed_hulls(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])
        .scaling(3.0).depth(4)
        .dashed_full_tree()
        .build()
        .draw_full();

    // Regular pentagon, non-centered
    let pentagon = move || EuclideanCanvas::builder()
        .fixed_hulls(vec![(0.2, 0.0), (0.8, 0.0), (1.0, 0.6), (0.5, 1.0), (0.0, 0.6)])
        .scaling(3.0).depth(3)
        .solid_full_tree()
        .draw_scaled_hulls()
        .build()
        .draw_full();

    // Regular hexagon
    let hexagon = move || EuclideanCanvas::builder()
        .fixed_hulls(vec![
            (0.5 - sqrt3_over_6, 0.0), (0.5 + sqrt3_over_6, 0.0), (1.0, 0.5),
            (0.5 + sqrt3_over_6, 1.0), (0.5 - sqrt3_over_6, 1.0), (0.0, 0.5),
        ])
        .scaling(3.5).depth(3)
        .dashed_full_tree()
        .draw_scaled_hulls()
        .build()
        .draw_full();

    // Vicset snowflake
    let vicset_snowflake = move || EuclideanCanvas::builder()
        .fixed_hulls(vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)])
        .scaling(3.1).depth(4)
        .solid_full_tree()
        .draw_scaled_dots()
        .build()
        .draw_full();

    // Hexaflake
    let hexaflake = move || EuclideanCanvas::builder()
        .fixed_hulls(vec![
            (0.5, 0.5),
            (0.5 - sqrt3_over_6, 0.0), (0.5 + sqrt3_over_6, 0.0), (1.0, 0.5),
            (0.5 + sqrt3_over_6, 1.0), (0.5 - sqrt3_over_6, 1.0), (0.0, 0.5),
        ])
        .scaling(3.0).depth(3)
        .draw_scaled_hulls()
        .build()
        .draw_full();

    let basic = view! {
        <Collapse title="Basic">

            <Flex>

                <ShapeCard class="euclidean"
                    title="Sierpinsky gasket"
                    shape=sierpinsky
                />

                <ShapeCard class="euclidean"
                    title="Rectangle"
                    shape=rectangle
                />

                <ShapeCard class="euclidean"
                    title="Pentagon"
                    shape=pentagon
                />
            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="Hexagon"
                    shape=hexagon
                />

                <ShapeCard class="euclidean"
                    title="Vicset snowflake"
                    shape=vicset_snowflake
                />

                <ShapeCard class="euclidean"
                    title="Hexaflake"
                    shape=hexaflake
                />

            </Flex>

        </Collapse>
    };

    let regular_poly = |sides: u32, scaling: f64, depth: isize| {
        let vec_digits = (0..sides).map(|d| {
            let angle = TAU * f64::from(d) / f64::from(sides);
            (angle.cos(), angle.sin())
        }).collect::<Vec<_>>();
        move || EuclideanCanvas::builder()
            .fixed_hulls(vec_digits.clone())
            .scaling(scaling).depth(depth)
            .solid_full_tree()
            .draw_scaled_hulls()
            .build()
            .draw_full()
    };

    let triangle = regular_poly(3, 2.01, 4);
    let rectangle = regular_poly(4, 2.2, 3);
    let pentagon = regular_poly(5, 2.6, 3);
    let hexagon = regular_poly(6, 3.0, 3);
    let septagon = regular_poly(7, 3.3, 3);
    let octagon = regular_poly(8, 3.5, 3);

    let regular = view! {
        <Collapse title="Regular">

            <Flex>

                <ShapeCard class="euclidean"
                    title="Triangle"
                    shape=triangle
                />

                <ShapeCard class="euclidean"
                    title="Rectangle"
                    shape=rectangle
                />

                <ShapeCard class="euclidean"
                    title="Pentagon"
                    shape=pentagon
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="Hexagon"
                    shape=hexagon
                />

                <ShapeCard class="euclidean"
                    title="Septagon"
                    shape=septagon
                />

                <ShapeCard class="euclidean"
                    title="Octagon"
                    shape=octagon
                />

            </Flex>

        </Collapse>
    };

    let colored_poly = |sides: u32, scaling: f64, depth: isize| {
        let vec_digits = (0..sides).map(|d| {
            let angle = TAU * f64::from(d) / f64::from(sides);
            (angle.cos(), angle.sin())
        }).collect::<Vec<_>>();
        let ones = EAdic::new_repeating(sides, vec![], vec![1]);
        let one_twos = EAdic::new_repeating(sides, vec![], vec![1, 2]);
        let two_ones = EAdic::new_repeating(sides, vec![], vec![2, 1]);
        let two_one = EAdic::new(sides, vec![2, 1]);
        move || EuclideanCanvas::builder()
            .fixed_hulls(vec_digits.clone())
            .scaling(scaling).depth(depth)
            .draw_scaled_hulls()
            .build()
            .draw_integers(&[ones.clone(), one_twos.clone(), two_ones.clone(), two_one.clone()])
    };

    let split_poly = |sides: u32, scaling: f64, depth: isize| {
        let vec_digits = (0..sides).map(|d| {
            let angle = TAU * f64::from(d) / f64::from(sides);
            (angle.cos(), angle.sin())
        }).collect::<Vec<_>>();
        let ones = EAdic::new_repeating(sides, vec![], vec![1]);
        let one_twos = EAdic::new_repeating(sides, vec![], vec![1, 2]);
        let two_ones = EAdic::new_repeating(sides, vec![], vec![2, 1]);
        let two_one = EAdic::new(sides, vec![2, 1]);
        move || EuclideanCanvas::builder()
            .fixed_hulls(vec_digits.clone())
            .scaling(scaling).depth(depth)
            .build()
            .draw_integers(&[ones.clone(), one_twos.clone(), two_ones.clone(), two_one.clone()])
    };

    let triangle = colored_poly(3, 2.01, 4);
    let pentagon = colored_poly(5, 2.6, 3);
    let septagon = colored_poly(7, 3.5, 3);

    let split_triangle = split_poly(3, 2.01, 8);
    let split_pentagon = split_poly(5, 2.6, 8);
    let split_septagon = split_poly(7, 3.5, 8);

    let colored = view! {
        <Collapse title="Colored">

            <Flex>

                <ShapeCard class="euclidean"
                    title="Triangle"
                    shape=triangle
                />

                <ShapeCard class="euclidean"
                    title="Pentagon"
                    shape=pentagon
                />

                <ShapeCard class="euclidean"
                    title="Septagon"
                    shape=septagon
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="Triangle"
                    shape=split_triangle
                />

                <ShapeCard class="euclidean"
                    title="Pentagon"
                    shape=split_pentagon
                />

                <ShapeCard class="euclidean"
                    title="Septagon"
                    shape=split_septagon
                />

            </Flex>

        </Collapse>
    };

    let treelike_vec_digits = |branches: u32| {
        (0..branches).map(|d| (
            2.0 * f64::from(d) / (f64::from(branches) - 1.0) - 1.0,
            1.0
        )).collect::<Vec<_>>()
    };
    let treelike = |branches: u32, scaling: f64, depth: isize| {
        let vec_digits = treelike_vec_digits(branches);
        move || EuclideanCanvas::builder()
            .fixed_hulls(vec_digits.clone())
            .scaling(scaling).depth(depth)
            .solid_full_tree()
            .build()
            .draw_full()
    };

    let binary_scaled2_5 = treelike(2, 2.5, 5);
    let binary_scaled2 = treelike(2, 2.0, 5);
    let binary_scaled1_1 = treelike(2, 1.1, 5);

    let ternary_scaled3_5 = treelike(3, 3.5, 4);
    let ternary_scaled3 = treelike(3, 3.0, 4);
    let quinary_scaled5_5 = treelike(5, 5.5, 3);

    let roots5 = ZAdic::roots_of_unity(5, 3).unwrap();
    let roots_of_unity_5 = move || {
        EuclideanCanvas::builder()
            .fixed_hulls(treelike_vec_digits(5))
            .scaling(5.5).depth(3)
            .solid_full_tree()
            .build()
            .draw_integers(roots5.roots())
    };
    let teich5 = ZAdic::teichmuller(5, 3).unwrap();
    let teichmuller_5 = move || {
        EuclideanCanvas::builder()
            .fixed_hulls(treelike_vec_digits(5))
            .scaling(5.5).depth(3)
            .solid_full_tree()
            .build()
            .draw_integers(teich5.roots())
    };
    let sqrt_two_7 = move || EuclideanCanvas::builder()
        .fixed_hulls(treelike_vec_digits(7))
        .scaling(7.5).depth(3)
        .solid_full_tree()
        .build()
        .draw_integers(&[EAdic::new(7, vec![3, 1, 2, 6]), EAdic::new(7, vec![4, 5, 4, 0])]);

    let roots5 = ZAdic::roots_of_unity(5, 8).unwrap();
    let split_roots_of_unity_5 = move || {
        EuclideanCanvas::builder()
            .fixed_hulls(treelike_vec_digits(5))
            .scaling(1.5).depth(8)
            .build()
            .draw_integers(roots5.roots())
    };
    let teich5 = ZAdic::teichmuller(5, 8).unwrap();
    let split_teichmuller_5 = move || {
        EuclideanCanvas::builder()
            .fixed_hulls(treelike_vec_digits(5))
            .scaling(1.5).depth(8)
            .build()
            .draw_integers(teich5.roots())
    };
    let split_sqrt_two_7 = move || EuclideanCanvas::builder()
        .fixed_hulls(treelike_vec_digits(7))
        .scaling(1.5).depth(8)
        .build()
        .draw_integers(&[EAdic::new(7, vec![3, 1, 2, 6, 1, 2, 1, 2]), EAdic::new(7, vec![4, 5, 4, 0, 5, 4, 5, 4])]);

    let treelike = view! {
        <Collapse title="Treelike">

            <Flex>

                <ShapeCard class="euclidean"
                    title="Binary, scaled 2.5"
                    shape=binary_scaled2_5
                />

                <ShapeCard class="euclidean"
                    title="Binary, scaled 2"
                    shape=binary_scaled2
                />

                <ShapeCard class="euclidean"
                    title="Binary, scaled 1.1"
                    shape=binary_scaled1_1
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="Ternary, scaled 3.5"
                    shape=ternary_scaled3_5
                />

                <ShapeCard class="euclidean"
                    title="Ternary, scaled 3"
                    shape=ternary_scaled3
                />

                <ShapeCard class="euclidean"
                    title="Quinary, scaled 5.5"
                    shape=quinary_scaled5_5
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="5-adic roots of unity"
                    shape=roots_of_unity_5
                />

                <ShapeCard class="euclidean"
                    title="5-adic Teichmuller characters"
                    shape=teichmuller_5
                />

                <ShapeCard class="euclidean"
                    title="7-adic sqrts of 2"
                    shape=sqrt_two_7
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="5-adic roots of unity (naive, split)"
                    shape=split_roots_of_unity_5
                />

                <ShapeCard class="euclidean"
                    title="5-adic Teichmuller characters (naive, split)"
                    shape=split_teichmuller_5
                />

                <ShapeCard class="euclidean"
                    title="7-adic sqrts of 2 (naive, split)"
                    shape=split_sqrt_two_7
                />

            </Flex>

        </Collapse>
    };

    // 3-adic
    let adic_3 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(3)
        .scaling(2.2).depth(5)
        .draw_scaled_hulls()
        .build()
        .draw_full();

    // 5-adic
    let adic_5 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(5)
        .scaling(2.8).depth(4)
        .draw_scaled_hulls()
        .build()
        .draw_full();

    // 7-adic
    let adic_7 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(7)
        .scaling(3.6).depth(3)
        .draw_scaled_hulls()
        .build()
        .draw_full();

    // 3-adic teichmuller characters
    let teich = ZAdic::teichmuller(3, 6).unwrap();
    let adic_3_teich = move || EuclideanCanvas::builder()
        .characteristic_p_adic(3)
        .scaling(2.2).depth(6)
        .draw_scaled_hulls()
        .build()
        .draw_integers(teich.roots());

    // 5-adic teichmuller characters
    let teich = ZAdic::teichmuller(5, 5).unwrap();
    let adic_5_teich = move || EuclideanCanvas::builder()
        .characteristic_p_adic(5)
        .scaling(2.8).depth(5)
        .draw_scaled_hulls()
        .build()
        .draw_integers(teich.roots());

    // 7-adic teichmuller characters
    let teich = ZAdic::teichmuller(7, 4).unwrap();
    let adic_7_teich = move || EuclideanCanvas::builder()
        .characteristic_p_adic(7)
        .scaling(3.6).depth(4)
        .draw_scaled_hulls()
        .build()
        .draw_integers(teich.roots());

    // 2-adic
    let adic_2 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(2)
        .scaling(2.0).depth(5)
        .draw_scaled_dots()
        .solid_full_tree()
        .draw_enclosing_disks(vec![0, 1, 2, 3])
        .build()
        .draw_full();

    // 3-adic teichmuller characters
    let teich = ZAdic::teichmuller(3, 10).unwrap();
    let split_adic_3_teich = move || EuclideanCanvas::builder()
        .characteristic_p_adic(3)
        .scaling(2.2).depth(10)
        .build()
        .draw_integers(teich.roots());

    // 5-adic teichmuller characters
    let teich = ZAdic::teichmuller(5, 10).unwrap();
    let split_adic_5_teich = move || EuclideanCanvas::builder()
        .characteristic_p_adic(5)
        .scaling(2.8).depth(10)
        .build()
        .draw_integers(teich.roots());

    // 7-adic teichmuller characters
    let teich = ZAdic::teichmuller(7, 10).unwrap();
    let split_adic_7_teich = move || EuclideanCanvas::builder()
        .characteristic_p_adic(7)
        .scaling(3.6).depth(10)
        .build()
        .draw_integers(teich.roots());

    // 2-adic teichmuller characters
    let teich = ZAdic::teichmuller(2, 5).unwrap();
    let adic_2_teich = move || EuclideanCanvas::builder()
        .characteristic_p_adic(2)
        .scaling(2.0).depth(5)
        .draw_scaled_dots()
        .build()
        .draw_integers(teich.roots());

    // 2-adic 0 + roots of unity
    // Solutions of x^3 - x = 0
    let poly = Polynomial::<ZAdic>::new_with_prime(2, vec![0, -1, 0, 1]);
    let roots = poly.variety(5).and_then(Variety::try_into_integer).unwrap();
    let adic_2_zero_and_unity = move || EuclideanCanvas::builder()
        .characteristic_p_adic(2)
        .scaling(2.0).depth(5)
        .draw_scaled_dots()
        .build()
        .draw_integers(roots.roots());

    // 2-adic depth = 0
    let adic_2_depth0 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(2)
        .scaling(2.1).depth(0)
        .draw_scaled_dots()
        .dashed_full_tree()
        .draw_enclosing_disks(vec![0, 1, 2, 3])
        .build()
        .draw_full();

    // 2-adic depth = 1
    let adic_2_depth1 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(2)
        .scaling(2.1).depth(1)
        .draw_scaled_dots()
        .dashed_full_tree()
        .draw_enclosing_disks(vec![0, 1, 2, 3])
        .build()
        .draw_full();

    // 2-adic depth = 2
    let adic_2_depth2 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(2)
        .scaling(2.1).depth(2)
        .draw_scaled_dots()
        .dashed_full_tree()
        .draw_enclosing_disks(vec![0, 1, 2, 3])
        .build()
        .draw_full();

    let characteristic = view! {
        <Collapse title="Characteristic p-adic">

            <Flex>

                <ShapeCard class="euclidean"
                    title="3-adic"
                    shape=adic_3
                />

                <ShapeCard class="euclidean"
                    title="5-adic"
                    shape=adic_5
                />

                <ShapeCard class="euclidean"
                    title="7-adic"
                    shape=adic_7
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="3-adic Teichmuller characters"
                    shape=adic_3_teich
                />

                <ShapeCard class="euclidean"
                    title="5-adic Teichmuller characters"
                    shape=adic_5_teich
                />

                <ShapeCard class="euclidean"
                    title="7-adic Teichmuller characters"
                    shape=adic_7_teich
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="3-adic Teichmuller characters (split)"
                    shape=split_adic_3_teich
                />

                <ShapeCard class="euclidean"
                    title="5-adic Teichmuller characters (split)"
                    shape=split_adic_5_teich
                />

                <ShapeCard class="euclidean"
                    title="7-adic Teichmuller characters (split)"
                    shape=split_adic_7_teich
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="2-adic"
                    shape=adic_2
                />

                <ShapeCard class="euclidean"
                    title="2-adic Teichmuller characters"
                    shape=adic_2_teich
                />

                <ShapeCard class="euclidean"
                    title="2-adic solutions of `x^3 - x = 0`"
                    shape=adic_2_zero_and_unity
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="2-adic depth 0"
                    shape=adic_2_depth0
                />

                <ShapeCard class="euclidean"
                    title="2-adic depth 1"
                    shape=adic_2_depth1
                />

                <ShapeCard class="euclidean"
                    title="2-adic depth 2"
                    shape=adic_2_depth2
                />

            </Flex>

        </Collapse>
    };

    let fractional_canvas = Signal::derive(move ||
        EuclideanCanvas::builder()
            .characteristic_p_adic(5)
            .scaling(2.5).depth(4)
            .build()
    );
    let fractional = view! {
        <Collapse title="Fractional">

            <Flex>
                <ShapeCard class="tree"
                    title="1/5 = ...000000.1"
                    shape=fractional_canvas.get().draw_number(&QAdic::new(EAdic::primed_from(5, 1), -1))
                />
                <ShapeCard class="tree"
                    title="-1/5 = ...444444.4"
                    shape=fractional_canvas.get().draw_number(&QAdic::new(EAdic::primed_from(5, -1), -1))
                />
                <ShapeCard class="tree"
                    title="-1/20 = ...111111.1"
                    shape=fractional_canvas.get().draw_number(&QAdic::<EAdic>::primed_from(5, Rational32::new(-1, 20)))
                />
                <ShapeCard class="tree"
                    title="-1/120 = ...101010.1"
                    shape=fractional_canvas.get().draw_number(&QAdic::<EAdic>::primed_from(5, Rational32::new(-1, 120)))
                />
                <ShapeCard class="tree"
                    title="-5/120 = ...010101.0"
                    shape=fractional_canvas.get().draw_number(&QAdic::<EAdic>::primed_from(5, Rational32::new(-5, 120)))
                />
            </Flex>

        </Collapse>
    };

    // 3-adic
    let directed_adic_3 = |direction: Direction, orientation: Orientation| {
        move || EuclideanCanvas::builder()
            .characteristic_p_adic(3)
            .scaling(2.2).depth(5)
            .draw_scaled_hulls()
            .direction(direction)
            .orientation(orientation)
            .build()
            .draw_integers(&[ZAdic::zero(3), ZAdic::one(3)])
    };
    // Sierpinsky
    let resized_sierpinsky = |direction: Direction, orientation: Orientation, resize_type: u32| {
        let sqrt2_over_2 = 2.0_f64.sqrt() * 0.5;
        move || {
            let builder = EuclideanCanvas::builder()
                .fixed_hulls(vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0 * sqrt2_over_2)])
                .scaling(2.1).depth(5)
                .draw_scaled_hulls()
                .direction(direction)
                .orientation(orientation);
            let canvas = match resize_type {
                0 => builder.no_resize().build(),
                1 => builder.resize_to_window().build(),
                2 => builder.resize_around_zero().build(),
                _ => builder.resize_to_window().build(),
            };
            canvas.draw_full()
        }
    };
    let directions = view! {
        <Collapse title="Direction, orientation, resize">

            <Flex>

                <ShapeCard class="euclidean"
                    title="Right CCW"
                    shape=directed_adic_3(Direction::Right, Orientation::CCW)
                />

                <ShapeCard class="euclidean"
                    title="Up CCW"
                    shape=directed_adic_3(Direction::Up, Orientation::CCW)
                />

                <ShapeCard class="euclidean"
                    title="Left CCW"
                    shape=directed_adic_3(Direction::Left, Orientation::CCW)
                />

                <ShapeCard class="euclidean"
                    title="Down CCW"
                    shape=directed_adic_3(Direction::Down, Orientation::CCW)
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="Right CW"
                    shape=directed_adic_3(Direction::Right, Orientation::CW)
                />

                <ShapeCard class="euclidean"
                    title="Up CW"
                    shape=directed_adic_3(Direction::Up, Orientation::CW)
                />

                <ShapeCard class="euclidean"
                    title="Left CW"
                    shape=directed_adic_3(Direction::Left, Orientation::CW)
                />

                <ShapeCard class="euclidean"
                    title="Down CW"
                    shape=directed_adic_3(Direction::Down, Orientation::CW)
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="Un-resized Sierpinsky"
                    shape=resized_sierpinsky(Direction::Right, Orientation::CCW, 0)
                />

                <ShapeCard class="euclidean"
                    title="Fit-to-window Sierpinsky"
                    shape=resized_sierpinsky(Direction::Right, Orientation::CCW, 1)
                />

                <ShapeCard class="euclidean"
                    title="Fit-around-zero Sierpinsky"
                    shape=resized_sierpinsky(Direction::Right, Orientation::CCW, 2)
                />

            </Flex>

            <Flex>

                <ShapeCard class="euclidean"
                    title="Un-resized, flipped, rotated Sierpinsky"
                    shape=resized_sierpinsky(Direction::Up, Orientation::CW, 0)
                />

                <ShapeCard class="euclidean"
                    title="Fit-to-window, flipped, rotated Sierpinsky"
                    shape=resized_sierpinsky(Direction::Up, Orientation::CW, 1)
                />

                <ShapeCard class="euclidean"
                    title="Fit-around-zero, flipped, rotated Sierpinsky"
                    shape=resized_sierpinsky(Direction::Up, Orientation::CW, 2)
                />

            </Flex>

        </Collapse>
    };

    let small_full_depth_input = signal(3isize);
    let small_full_depth_slider = (
        Signal::derive(move || small_full_depth_input.0.get().to_f64().unwrap()),
        SignalSetter::<f64>::map(move |new_val| small_full_depth_input.1.set(new_val.round().to_isize().unwrap())),
    );
    let min_small_full_depth = 1;
    let min_small_full_depth_slider = 1.0;
    let max_small_full_depth = 5;
    let max_small_full_depth_slider = 5.0;

    let small_full_scale_input = signal(2.5);
    let small_full_scale_slider = (
        Signal::derive(move || small_full_scale_input.0.get()),
        SignalSetter::<f64>::map(move |new_val| small_full_scale_input.1.set(new_val))
    );
    let min_small_full_scale = 0.0;
    let max_small_full_scale = 5.0;
    let step_small_full_scale = 0.01;
    let small_full_scale_format = move |v: f64| format!("{v:.2}");

    let teich3 = Signal::derive(move || ZAdic::teichmuller(3, 10).unwrap().into_roots().collect::<Vec<_>>());
    let full_euclidean3 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(3)
        .scaling(small_full_scale_slider.0.get())
        .depth(small_full_depth_input.0.get())
        .draw_scaled_hulls()
        .build()
        .draw_integers(&teich3.get());
    let teich5 = Signal::derive(move || ZAdic::teichmuller(5, 10).unwrap().into_roots().collect::<Vec<_>>());
    let full_euclidean5 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(5)
        .scaling(small_full_scale_slider.0.get())
        .depth(small_full_depth_input.0.get())
        .draw_scaled_hulls()
        .build()
        .draw_integers(&teich5.get());
    let teich7 = Signal::derive(move || ZAdic::teichmuller(7, 10).unwrap().into_roots().collect::<Vec<_>>());
    let full_euclidean7 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(7)
        .scaling(small_full_scale_slider.0.get())
        .depth(small_full_depth_input.0.get())
        .draw_scaled_hulls()
        .build()
        .draw_integers(&teich7.get());

    let big_full_depth_input = signal(2isize);
    let big_full_depth_slider = (
        Signal::derive(move || big_full_depth_input.0.get().to_f64().unwrap()),
        SignalSetter::<f64>::map(move |new_val| big_full_depth_input.1.set(new_val.round().to_isize().unwrap())),
    );
    let min_big_full_depth = 1;
    let min_big_full_depth_slider = 1.0;
    let max_big_full_depth = 5;
    let max_big_full_depth_slider = 5.0;

    let big_full_scale_input = signal(2.5);
    let big_full_scale_slider = (
        Signal::derive(move || big_full_scale_input.0.get()),
        SignalSetter::<f64>::map(move |new_val| big_full_scale_input.1.set(new_val))
    );
    let min_big_full_scale = 0.0;
    let max_big_full_scale = 5.0;
    let step_big_full_scale = 0.01;
    let big_full_scale_format = move |v: f64| format!("{v:.2}");

    let teich11 = Signal::derive(move || ZAdic::teichmuller(11, 10).unwrap().into_roots().collect::<Vec<_>>());
    let full_euclidean11 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(11)
        .scaling(big_full_scale_slider.0.get())
        .depth(big_full_depth_input.0.get())
        .draw_scaled_hulls()
        .build()
        .draw_integers(&teich11.get());
    let teich13 = Signal::derive(move || ZAdic::teichmuller(13, 10).unwrap().into_roots().collect::<Vec<_>>());
    let full_euclidean13 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(13)
        .scaling(big_full_scale_slider.0.get())
        .depth(big_full_depth_input.0.get())
        .draw_scaled_hulls()
        .build()
        .draw_integers(&teich13.get());
    let teich17 = Signal::derive(move || ZAdic::teichmuller(17, 10).unwrap().into_roots().collect::<Vec<_>>());
    let full_euclidean17 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(17)
        .scaling(big_full_scale_slider.0.get())
        .depth(big_full_depth_input.0.get())
        .draw_scaled_hulls()
        .build()
        .draw_integers(&teich17.get());

    let small_split_depth_input = signal(8isize);
    let small_split_depth_slider = (
        Signal::derive(move || small_split_depth_input.0.get().to_f64().unwrap()),
        SignalSetter::<f64>::map(move |new_val| small_split_depth_input.1.set(new_val.round().to_isize().unwrap())),
    );
    let min_small_split_depth = 1;
    let min_small_split_depth_slider = 1.0;
    let max_small_split_depth = 40;
    let max_small_split_depth_slider = 40.0;

    let small_split_scale_input = signal(2.5);
    let small_split_scale_slider = (
        Signal::derive(move || small_split_scale_input.0.get()),
        SignalSetter::<f64>::map(move |new_val| small_split_scale_input.1.set(new_val))
    );
    let min_small_split_scale = 0.0;
    let max_small_split_scale = 5.0;
    let step_small_split_scale = 0.01;
    let small_split_scale_format = move |v: f64| format!("{v:.2}");

    let teich3 = Signal::derive(move || ZAdic::teichmuller(3, small_split_depth_input.0.get().try_into().unwrap()).unwrap().into_roots().collect::<Vec<_>>());
    let split_euclidean3 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(3)
        .scaling(small_split_scale_slider.0.get())
        .depth(small_split_depth_input.0.get())
        .build()
        .draw_integers(&teich3.get());
    let teich5 = Signal::derive(move || ZAdic::teichmuller(5, small_split_depth_input.0.get().try_into().unwrap()).unwrap().into_roots().collect::<Vec<_>>());
    let split_euclidean5 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(5)
        .scaling(small_split_scale_slider.0.get())
        .depth(small_split_depth_input.0.get())
        .build()
        .draw_integers(&teich5.get());
    let teich7 = Signal::derive(move || ZAdic::teichmuller(7, small_split_depth_input.0.get().try_into().unwrap()).unwrap().into_roots().collect::<Vec<_>>());
    let split_euclidean7 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(7)
        .scaling(small_split_scale_slider.0.get())
        .depth(small_split_depth_input.0.get())
        .build()
        .draw_integers(&teich7.get());


    let big_split_depth_input = signal(8isize);
    let big_split_depth_slider = (
        Signal::derive(move || big_split_depth_input.0.get().to_f64().unwrap()),
        SignalSetter::<f64>::map(move |new_val| big_split_depth_input.1.set(new_val.round().to_isize().unwrap())),
    );
    let min_big_split_depth = 1;
    let min_big_split_depth_slider = 1.0;
    let max_big_split_depth = 40;
    let max_big_split_depth_slider = 40.0;

    let big_split_scale_input = signal(2.5);
    let big_split_scale_slider = (
        Signal::derive(move || big_split_scale_input.0.get()),
        SignalSetter::<f64>::map(move |new_val| big_split_scale_input.1.set(new_val))
    );
    let min_big_split_scale = 0.0;
    let max_big_split_scale = 5.0;
    let step_big_split_scale = 0.01;
    let big_split_scale_format = move |v: f64| format!("{v:.2}");

    let teich11 = Signal::derive(move || ZAdic::teichmuller(11, big_split_depth_input.0.get().try_into().unwrap()).unwrap().into_roots().collect::<Vec<_>>());
    let split_euclidean11 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(11)
        .scaling(big_split_scale_slider.0.get())
        .depth(big_split_depth_input.0.get())
        .build()
        .draw_integers(&teich11.get());
    let teich13 = Signal::derive(move || ZAdic::teichmuller(13, big_split_depth_input.0.get().try_into().unwrap()).unwrap().into_roots().collect::<Vec<_>>());
    let split_euclidean13 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(13)
        .scaling(big_split_scale_slider.0.get())
        .depth(big_split_depth_input.0.get())
        .build()
        .draw_integers(&teich13.get());
    let teich17 = Signal::derive(move || ZAdic::teichmuller(17, big_split_depth_input.0.get().try_into().unwrap()).unwrap().into_roots().collect::<Vec<_>>());
    let split_euclidean17 = move || EuclideanCanvas::builder()
        .characteristic_p_adic(17)
        .scaling(big_split_scale_slider.0.get())
        .depth(big_split_depth_input.0.get())
        .build()
        .draw_integers(&teich17.get());

    let adjustable = view! {
        <Collapse title="Adjustable Euclidean">

            <Collapse title="Small primes, full Euclideans">

                <Flex vertical=true style="padding: 20px">
                    <SpinButton<isize>
                        value=small_full_depth_input
                        step_page=1 min=min_small_full_depth max=max_small_full_depth
                    />
                    <Slider
                        value=small_full_depth_slider
                        min=min_small_full_depth_slider max=max_small_full_depth_slider
                    />

                    <SpinButton<f64>
                        value=small_full_scale_input
                        step_page=step_small_full_scale min=min_small_full_scale max=max_small_full_scale
                        format=small_full_scale_format
                    />
                    <Slider
                        value=small_full_scale_slider
                        min=min_small_full_scale max=max_small_full_scale
                        step=step_small_full_scale show_stops=false
                    >
                        <SliderLabel value=0.0>
                            "0"
                        </SliderLabel>
                        <SliderLabel value=1.0>
                            "1"
                        </SliderLabel>
                        <SliderLabel value=2.0>
                            "2"
                        </SliderLabel>
                        <SliderLabel value=3.0>
                            "3"
                        </SliderLabel>
                        <SliderLabel value=4.0>
                            "4"
                        </SliderLabel>
                        <SliderLabel value=5.0>
                            "5"
                        </SliderLabel>
                    </Slider>
                </Flex>

                <Flex>

                    <ShapeCard class="euclidean"
                        title="3-adic Euclidean"
                        shape=full_euclidean3
                    />

                    <ShapeCard class="euclidean"
                        title="5-adic Euclidean"
                        shape=full_euclidean5
                    />

                    <ShapeCard class="euclidean"
                        title="7-adic Euclidean"
                        shape=full_euclidean7
                    />

                </Flex>

            </Collapse>

            <Collapse title="Larger primes, full Euclideans">

                <Flex vertical=true style="padding: 20px">
                    <SpinButton<isize>
                        value=big_full_depth_input
                        step_page=1 min=min_big_full_depth max=max_big_full_depth
                    />
                    <Slider
                        value=big_full_depth_slider
                        min=min_big_full_depth_slider max=max_big_full_depth_slider
                    />

                    <SpinButton<f64>
                        value=big_full_scale_input
                        step_page=step_big_full_scale min=min_big_full_scale max=max_big_full_scale
                        format=big_full_scale_format
                    />
                    <Slider
                        value=big_full_scale_slider
                        min=min_big_full_scale max=max_big_full_scale
                        step=step_big_full_scale show_stops=false
                    >
                        <SliderLabel value=0.0>
                            "0"
                        </SliderLabel>
                        <SliderLabel value=1.0>
                            "1"
                        </SliderLabel>
                        <SliderLabel value=2.0>
                            "2"
                        </SliderLabel>
                        <SliderLabel value=3.0>
                            "3"
                        </SliderLabel>
                        <SliderLabel value=4.0>
                            "4"
                        </SliderLabel>
                        <SliderLabel value=5.0>
                            "5"
                        </SliderLabel>
                    </Slider>
                </Flex>

                <Flex>

                    <ShapeCard class="euclidean"
                        title="11-adic Euclidean"
                        shape=full_euclidean11
                    />

                    <ShapeCard class="euclidean"
                        title="13-adic Euclidean"
                        shape=full_euclidean13
                    />

                    <ShapeCard class="euclidean"
                        title="17-adic Euclidean"
                        shape=full_euclidean17
                    />

                </Flex>

            </Collapse>

            <Collapse title="Small primes, split Euclideans">

                <Flex vertical=true style="padding: 20px">
                    <SpinButton<isize>
                        value=small_split_depth_input
                        step_page=1 min=min_small_split_depth max=max_small_split_depth
                    />
                    <Slider
                        value=small_split_depth_slider
                        min=min_small_split_depth_slider max=max_small_split_depth_slider
                    />

                    <SpinButton<f64>
                        value=small_split_scale_input
                        step_page=step_small_split_scale min=min_small_split_scale max=max_small_split_scale
                        format=small_split_scale_format
                    />
                    <Slider
                        value=small_split_scale_slider
                        min=min_small_split_scale max=max_small_split_scale
                        step=step_small_split_scale show_stops=false
                    >
                        <SliderLabel value=0.0>
                            "0"
                        </SliderLabel>
                        <SliderLabel value=1.0>
                            "1"
                        </SliderLabel>
                        <SliderLabel value=2.0>
                            "2"
                        </SliderLabel>
                        <SliderLabel value=3.0>
                            "3"
                        </SliderLabel>
                        <SliderLabel value=4.0>
                            "4"
                        </SliderLabel>
                        <SliderLabel value=5.0>
                            "5"
                        </SliderLabel>
                    </Slider>
                </Flex>

                <Flex>

                    <ShapeCard class="euclidean"
                        title="3-adic Euclidean"
                        shape=split_euclidean3
                    />

                    <ShapeCard class="euclidean"
                        title="5-adic Euclidean"
                        shape=split_euclidean5
                    />

                    <ShapeCard class="euclidean"
                        title="7-adic Euclidean"
                        shape=split_euclidean7
                    />

                </Flex>

            </Collapse>

            <Collapse title="Larger primes, split Euclideans">

                <Flex vertical=true style="padding: 20px">
                    <SpinButton<isize>
                        value=big_split_depth_input
                        step_page=1 min=min_big_split_depth max=max_big_split_depth
                    />
                    <Slider
                        value=big_split_depth_slider
                        min=min_big_split_depth_slider max=max_big_split_depth_slider
                    />

                    <SpinButton<f64>
                        value=big_split_scale_input
                        step_page=step_big_split_scale min=min_big_split_scale max=max_big_split_scale
                        format=big_split_scale_format
                    />
                    <Slider
                        value=big_split_scale_slider
                        min=min_big_split_scale max=max_big_split_scale
                        step=step_big_split_scale show_stops=false
                    >
                        <SliderLabel value=0.0>
                            "0"
                        </SliderLabel>
                        <SliderLabel value=1.0>
                            "1"
                        </SliderLabel>
                        <SliderLabel value=2.0>
                            "2"
                        </SliderLabel>
                        <SliderLabel value=3.0>
                            "3"
                        </SliderLabel>
                        <SliderLabel value=4.0>
                            "4"
                        </SliderLabel>
                        <SliderLabel value=5.0>
                            "5"
                        </SliderLabel>
                    </Slider>
                </Flex>

                <Flex>

                    <ShapeCard class="euclidean"
                        title="11-adic Euclidean"
                        shape=split_euclidean11
                    />

                    <ShapeCard class="euclidean"
                        title="13-adic Euclidean"
                        shape=split_euclidean13
                    />

                    <ShapeCard class="euclidean"
                        title="17-adic Euclidean"
                        shape=split_euclidean17
                    />

                </Flex>

            </Collapse>

        </Collapse>
    };

    let rts_a = |p: u32, a: i32, power: u32, depth: isize| {
        let ea = EAdic::primed_from(p, a);
        ea.nth_root(power, depth.try_into().unwrap()).unwrap()
    };
    let euc_rts = |p: u32, scale: f64, variety: &Variety<ZAdic>, depth: isize| {
        let roots = variety.roots().collect::<Vec<_>>();
        EuclideanCanvas::builder()
            .characteristic_p_adic(p)
            .scaling(scale).depth(depth)
            .direction(Direction::Up).orientation(Orientation::CW)
            .resize_around_zero()
            .build()
            .draw_integers(roots)
    };

    let depth = 64;

    let sqrt = rts_a(7, 2, 2, depth);
    let sqrt1 = sqrt.clone();
    let esqrt1 = move || euc_rts(7, 1.1, &sqrt1, depth);
    let sqrt2 = sqrt.clone();
    let esqrt2 = move || euc_rts(7, 1.01, &sqrt2, depth);
    let sqrt3 = sqrt.clone();
    let esqrt3 = move || euc_rts(7, 1.001, &sqrt3, depth);

    let qtrt = rts_a(7, 2, 4, depth);
    let qtrt1 = qtrt.clone();
    let eqtrt1 = move || euc_rts(7, 1.1, &qtrt1, depth);
    let qtrt2 = qtrt.clone();
    let eqtrt2 = move || euc_rts(7, 1.01, &qtrt2, depth);
    let qtrt3 = qtrt.clone();
    let eqtrt3 = move || euc_rts(7, 1.001, &qtrt3, depth);

    // Irrational roots of unity: roots of `x^4 + x^2 + 1 = 0`
    let sxrt = Polynomial::<EAdic>::new_with_prime(7, vec![1, 0, 1, 0, 1]).variety(depth.to_isize().unwrap()).unwrap().try_into_integer().unwrap();
    let sxrt1 = sxrt.clone();
    let esxrt1 = move || euc_rts(7, 1.1, &sxrt1, depth);
    let sxrt2 = sxrt.clone();
    let esxrt2 = move || euc_rts(7, 1.01, &sxrt2, depth);
    let sxrt3 = sxrt.clone();
    let esxrt3 = move || euc_rts(7, 1.001, &sxrt3, depth);

    let calculations = view! {
        <Collapse title="Calculations">

            <Flex>
                <ShapeCard class="euclidean"
                    title="7-adic sqrt(2) 1.1"
                    shape=esqrt1
                />
                <ShapeCard class="euclidean"
                    title="7-adic sqrt(2) 1.01"
                    shape=esqrt2
                />
                <ShapeCard class="euclidean"
                    title="7-adic sqrt(2) 1.001"
                    shape=esqrt3
                />
            </Flex>

            <Flex>
                <ShapeCard class="euclidean"
                    title="7-adic qtrt(2) 1.1"
                    shape=eqtrt1
                />
                <ShapeCard class="euclidean"
                    title="7-adic qtrt(2) 1.01"
                    shape=eqtrt2
                />
                <ShapeCard class="euclidean"
                    title="7-adic qtrt(2) 1.001"
                    shape=eqtrt3
                />
            </Flex>

            <Flex>
                <ShapeCard class="euclidean"
                    title="7-adic irrational sxrt(1) 1.1"
                    shape=esxrt1
                />
                <ShapeCard class="euclidean"
                    title="7-adic irrational sxrt(1) 1.01"
                    shape=esxrt2
                />
                <ShapeCard class="euclidean"
                    title="7-adic irrational sxrt(1) 1.001"
                    shape=esxrt3
                />
            </Flex>

        </Collapse>
    };

    view! {
        <section class="boxed-section">
            {basic}
            {regular}
            {colored}
            {treelike}
            {characteristic}
            {fractional}
            {directions}
            {adjustable}
            {calculations}
        </section>
    }

}
