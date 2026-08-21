use std::time::Duration;
use leptos::{prelude::*, reactive::wrappers::write::SignalSetter};
use thaw::{Checkbox, Flex, Select};

use adic_shape::adic::{
    traits::{AdicPrimitive, PrimedFrom},
    EAdic,
};
use adic_shape::{
    animation::{AnimationOptions, Frame, FrameReel},
    leptos::{AnimatedShapeCard, Collapse},
    shape::{AdicCanvas, ClockCanvas, TreeCanvas},
};


#[component]
pub fn AnimateSuite() -> impl IntoView {

    let clock_canvas = ClockCanvas::builder().base(5).depth(6).build();
    let animated_clock_frame_reel = FrameReel::new(
        (0..25).map(|num| {
            let shape = clock_canvas.draw_integer(&EAdic::primed_from(5, num));
            Frame::from((
                num,
                shape,
                num.to_string()
            ))
        }).collect::<Vec<_>>(),
        25
    );

    let tree_canvas = TreeCanvas::builder().base(5).depth(6).build();
    let animated_tree_frame_reel = FrameReel::new(
        (0..25).map(|num| {
            let shape = tree_canvas.draw_integer(&EAdic::primed_from(5, num));
            Frame::from((
                num,
                shape,
                num.to_string()
            ))
        }).collect::<Vec<_>>(),
        25
    );

    let zero = EAdic::zero(5);
    let neg_1_4 = EAdic::new_repeating(5, vec![], vec![1]);
    let neg_1_2 = neg_1_4.clone() + neg_1_4.clone();
    let neg_3_4 = neg_1_2.clone() + neg_1_4.clone();
    let neg_1 = neg_1_2.clone() + neg_1_2.clone();
    let num_digits = 6;

    let clock_canvas = ClockCanvas::builder().base(5).depth(num_digits).build();
    let clocks_neg_1_4 = FrameReel::simple_linear_labelled([
        (clock_canvas.draw_integer(&zero), "0"),
        (clock_canvas.draw_integer(&neg_1_4), "-1/4"),
        (clock_canvas.draw_integer(&neg_1_2), "-1/2"),
        (clock_canvas.draw_integer(&neg_3_4), "-3/4"),
        (clock_canvas.draw_integer(&neg_1), "-1"),
    ]);
    let tree_canvas = TreeCanvas::builder().base(5).depth(num_digits).build();
    let trees_neg_1_4 = FrameReel::simple_linear_labelled([
        (tree_canvas.draw_integer(&zero), "-0"),
        (tree_canvas.draw_integer(&neg_1_4), "-1/4"),
        (tree_canvas.draw_integer(&neg_1_2), "-1/2"),
        (tree_canvas.draw_integer(&neg_3_4), "-3/4"),
        (tree_canvas.draw_integer(&neg_1), "-1"),
    ]);

    let basic = view! {
        <Collapse title="Basic">

            <Flex>

                <AnimatedShapeCard class="clock"
                    title="0 -> 24"
                    shape_reel=animated_clock_frame_reel
                />

                <AnimatedShapeCard class="tree"
                    title="0 -> 24"
                    shape_reel=animated_tree_frame_reel
                />

            </Flex>

            <Flex>

                <AnimatedShapeCard class="clock"
                    title="0, -1/4, -1/2, -3/4, -1"
                    shape_reel=clocks_neg_1_4
                />

                <AnimatedShapeCard class="tree"
                    title="0, -1/4, -1/2, -3/4, -1"
                    shape_reel=trees_neg_1_4
                />

            </Flex>

        </Collapse>
    };

    let zero = EAdic::zero(5);
    let pos_1_4 = EAdic::new_repeating(5, vec![4], vec![3]);
    let pos_1_2 = pos_1_4.clone() + pos_1_4.clone();
    let pos_3_4 = pos_1_2.clone() + pos_1_4.clone();
    let pos_1 = pos_1_2.clone() + pos_1_2.clone();
    let num_digits = 6;

    let clock_canvas = ClockCanvas::builder().base(5).depth(num_digits).build();
    let clocks_1_4 = FrameReel::simple_linear_labelled([
        (clock_canvas.draw_integer(&zero), "0"),
        (clock_canvas.draw_integer(&pos_1_4), "1/4"),
        (clock_canvas.draw_integer(&pos_1_2), "1/2"),
        (clock_canvas.draw_integer(&pos_3_4), "3/4"),
        (clock_canvas.draw_integer(&pos_1), "1"),
    ]);
    let tree_canvas = TreeCanvas::builder().base(5).depth(num_digits).build();
    let trees_1_4 = FrameReel::simple_linear_labelled([
        (tree_canvas.draw_integer(&zero), "0"),
        (tree_canvas.draw_integer(&pos_1_4), "1/4"),
        (tree_canvas.draw_integer(&pos_1_2), "1/2"),
        (tree_canvas.draw_integer(&pos_3_4), "next frame intentionally blank"),
        (tree_canvas.draw_integer(&pos_1), ""),
    ]);


    // Copied from AnimatedShapeOptions
    let options = RwSignal::new(AnimationOptions::default());

    macro_rules! map_signal {
        ( $option_attr:ident ) => {
            (
                Signal::derive(move || options.get().$option_attr),
                SignalSetter::map(move |new_val| options.write().$option_attr = new_val)
            )
        }
    }

    let should_auto_start = map_signal!(should_auto_start);
    let should_loop = map_signal!(should_loop);
    let show_slider = map_signal!(show_slider);
    let show_play_reset = map_signal!(show_play_reset);
    let show_skip = map_signal!(show_skip);
    let show_frame_label = map_signal!(show_frame_label);

    let tick_time_select = RwSignal::new("2 fps".to_string());
    let tick_time = (
        Signal::derive(move || options.get().tick_time),
        SignalSetter::<Duration>::map(move |new_val| options.write().tick_time = new_val)
    );
    Effect::watch(
        move || tick_time_select.get(),
        move |select, _, _| {
            tick_time.1.set(match select.as_ref() {
                "20 fps" => Duration::from_millis(50),
                "10 fps" => Duration::from_millis(100),
                "5 fps" => Duration::from_millis(200),
                "2 fps" => Duration::from_millis(500),
                "1 fps" => Duration::from_secs(1),
                _ => Duration::from_secs(1),
            });
        },
        true
    );

    let toggle_options = view! {
        <Collapse title="Options">

            <Flex>
                <Select value=tick_time_select default_value="2 fps">
                    <option>"20 fps"</option>
                    <option>"10 fps"</option>
                    <option>"5 fps"</option>
                    <option>"2 fps"</option>
                    <option>"1 fps"</option>
                </Select>
                <Checkbox checked=should_auto_start label="Auto-start"/>
                <Checkbox checked=should_loop label="Looping"/>
            </Flex>
            <Flex>
                <Checkbox checked=show_frame_label label="Show frame label"/>
                <Checkbox checked=show_slider label="Show slider"/>
                <Checkbox checked=show_play_reset label="Show play/reset"/>
                <Checkbox checked=show_skip label="Show skip"/>
            </Flex>

            <Flex>

                <AnimatedShapeCard class="clock"
                    title="0, 1/4, 1/2, 3/4, 1"
                    shape_reel=clocks_1_4
                    options=options
                />

                <AnimatedShapeCard class="tree"
                    title="0, 1/4, 1/2, 3/4, 1"
                    shape_reel=trees_1_4
                    options=options
                />

            </Flex>
        </Collapse>
    };

    view! {
        <section class="boxed-section">
            <h3>"Animation examples"</h3>
            {basic}
            {toggle_options}
        </section>
    }

}
