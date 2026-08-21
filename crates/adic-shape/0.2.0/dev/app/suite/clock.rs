use leptos::prelude::*;
use num::{ToPrimitive, Rational32};
use thaw::{Flex, FlexAlign, Slider, SpinButton};

use adic_shape::adic::{
    traits::{AdicInteger, PrimedFrom, TryPrimedFrom},
    num_adic::{MAdic, PowAdic},
    EAdic, QAdic, ZAdic,
};
use adic_shape::{
    error::AdicShapeError,
    leptos::{Collapse, ShapeCard, ShapeComponent},
    shape::{AdicCanvas, ClockCanvas, ClockShape},
};


#[component]
pub fn ClockSuite() -> impl IntoView {

    let basic_canvas = Signal::derive(
        move || ClockCanvas::builder().base(5).depth(6).show_val_circles(true).build()
    );
    let basic = view! {
        <Collapse title="Basic">

            <Flex>
                <ShapeCard class="clock"
                    title="1 = ...000001"
                    shape=basic_canvas.get().draw_integer(&EAdic::primed_from(5, 1))
                />
                <ShapeCard class="clock"
                    title="-1 = ...444444"
                    shape=basic_canvas.get().draw_integer(&EAdic::primed_from(5, -1))
                />
                <ShapeCard class="clock"
                    title="-1/4 = ...111111"
                    shape=basic_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-1, 4)).unwrap())
                />
                <ShapeCard class="clock"
                    title="-1/24 = ...010101"
                    shape=basic_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-1, 24)).unwrap())
                />
                <ShapeCard class="clock"
                    title="-5/24 = ...101010"
                    shape=basic_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-5, 24)).unwrap())
                />
            </Flex>

        </Collapse>
    };

    let fractional_canvas = Signal::derive(
        move || ClockCanvas::builder().base(5).depth(6).build()
    );
    let fractional = view! {
        <Collapse title="Fractional">

            <Flex>
                <ShapeCard class="clock"
                    title="1/5 = ...000000.1"
                    shape=fractional_canvas.get().draw_number(&QAdic::new(EAdic::primed_from(5, 1), -1))
                />
                <ShapeCard class="clock"
                    title="-1/5 = ...444444.4"
                    shape=fractional_canvas.get().draw_number(&QAdic::new(EAdic::primed_from(5, -1), -1))
                />
                <ShapeCard class="clock"
                    title="-1/20 = ...111111.1"
                    shape=fractional_canvas.get().draw_number(&QAdic::<EAdic>::primed_from(5, Rational32::new(-1, 20)))
                />
                <ShapeCard class="clock"
                    title="-1/120 = ...101010.1"
                    shape=fractional_canvas.get().draw_number(&QAdic::<EAdic>::primed_from(5, Rational32::new(-1, 120)))
                />
                <ShapeCard class="clock"
                    title="-5/120 = ...010101.0"
                    shape=fractional_canvas.get().draw_number(&QAdic::<EAdic>::primed_from(5, Rational32::new(-5, 120)))
                />
            </Flex>

        </Collapse>
    };

    let comp_vs_card_canvas = Signal::derive(
        move || ClockCanvas::builder().base(5).depth(6).show_val_circles(true).build()
    );
    let component_vs_card = view! {
        <Collapse title="Component vs Card">
            <h2>"Component"</h2>
            <ShapeComponent class="clock"
                shape=comp_vs_card_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-5, 24)).unwrap())
            />
            <h2>"Card"</h2>
            <ShapeCard class="clock"
                title="Clock title message"
                shape=comp_vs_card_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-5, 24)).unwrap())
            />
        </Collapse>
    };

    let num_digits_input = signal(12usize);
    let num_digits_slider = (
        Signal::derive(move || num_digits_input.0.get().to_f64().unwrap()),
        SignalSetter::<f64>::map(move |new_val| num_digits_input.1.set(new_val.round().to_usize().unwrap()))
    );
    let min_digits = 1;
    let min_digits_slider = 1.0;
    let max_digits = 100;
    let max_digits_slider = 100.0;

    let two = Signal::derive(move || EAdic::primed_from(7, 2));
    let sqrt_twos = Signal::derive(move || two.get().nth_root(2, num_digits_input.0.get()).unwrap().into_roots().collect::<Vec<_>>());

    let clock_canvas = Signal::derive(
        move || ClockCanvas::builder()
            .base(7)
            .depth(num_digits_input.0.get().try_into().unwrap())
            .build()
    );
    let clock_7_2 = move || clock_canvas.get().draw_integer(&two.get());
    let clock_7_first_sqrt_2 = move || clock_canvas.get().draw_integer(&sqrt_twos.get()[0]);
    let clock_7_second_sqrt_2 = move || clock_canvas.get().draw_integer(&sqrt_twos.get()[1]);
    let sqrt2_7adic = view! {
        <Collapse title="\\sqrt{2} in the 7-adics">

            <Flex vertical=true>
                <SpinButton<usize>
                    value=num_digits_input
                    step_page=1 min=min_digits max=max_digits
                />
                <Slider
                    value=num_digits_slider
                    min=min_digits_slider max=max_digits_slider
                />
            </Flex>

            <Flex align=FlexAlign::Center>

                <ShapeCard class="clock"
                    title="2"
                    shape=clock_7_2
                />

                <ShapeCard class="clock"
                    title="1st \\sqrt{2}"
                    shape=clock_7_first_sqrt_2
                />

                <ShapeCard class="clock"
                    title="2nd \\sqrt{2}"
                    shape=clock_7_second_sqrt_2
                />

            </Flex>

        </Collapse>
    };

    let sqrt7_canvas = Signal::derive(
        move || ClockCanvas::builder().base(3).depth(20).build()
    );
    let sqrt7_3adic = view! {
        <Collapse title="\\sqrt{7} in the 3-adics">

            <Flex>
                <ShapeCard class="clock"
                    title="Root 1"
                    shape=sqrt7_canvas.get().draw_integer(
                        &ZAdic::new_approx(3, 20, vec![1, 1, 1, 0, 2, 0, 0, 2, 1, 1, 2, 0, 2, 2, 2, 1, 0, 2, 1, 2])
                    )
                />
                <ShapeCard class="clock"
                    title="Root 2"
                    shape=sqrt7_canvas.get().draw_integer(
                        &ZAdic::new_approx(3, 20, vec![2, 1, 1, 2, 0, 2, 2, 0, 1, 1, 0, 2, 0, 0, 0, 1, 2, 0, 1, 0])
                    )
                />
            </Flex>

        </Collapse>
    };

    let unity3_canvas = Signal::derive(
        move || ClockCanvas::builder().base(3).depth(20).build()
    );
    let unity5_canvas = Signal::derive(
        move || ClockCanvas::builder().base(5).depth(20).build()
    );
    let unity7_canvas = Signal::derive(
        move || ClockCanvas::builder().base(7).depth(20).build()
    );
    let roots_of_unity = view! {
        <Collapse title="Roots of unity">

            <h2>"3-adic"</h2>

            <Flex>
                <ShapeCard class="clock"
                    shape=unity3_canvas.get().draw_integer(
                        &ZAdic::new_approx(3, 20, vec![1])
                    )
                />
                <ShapeCard class="clock"
                    shape=unity3_canvas.get().draw_integer(
                        &ZAdic::new_approx(3, 20, vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2])
                    )
                />
            </Flex>

            <h2>"5-adic"</h2>

            <Flex>
                <ShapeCard class="clock"
                    shape=unity5_canvas.get().draw_integer(
                        &ZAdic::new_approx(5, 20, vec![1])
                    )
                />
                <ShapeCard class="clock"
                    shape=unity5_canvas.get().draw_integer(
                        &ZAdic::new_approx(5, 20, vec![2, 1, 2, 1, 3, 4, 2, 3, 0, 3, 2, 2, 0, 4, 1, 3, 2, 4, 0, 4])
                    )
                />
                <ShapeCard class="clock"
                    shape=unity5_canvas.get().draw_integer(
                        &ZAdic::new_approx(5, 20, vec![3, 3, 2, 3, 1, 0, 2, 1, 4, 1, 2, 2, 4, 0, 3, 1, 2, 0, 4, 0])
                    )
                />
                <ShapeCard class="clock"
                    shape=unity5_canvas.get().draw_integer(
                        &ZAdic::new_approx(5, 20, vec![4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4])
                    )
                />
            </Flex>

            <h2>"7-adic"</h2>

            <Flex>
                <ShapeCard class="clock"
                    shape=unity7_canvas.get().draw_integer(
                        &ZAdic::new_approx(7, 20, vec![1])
                    )
                />
                <ShapeCard class="clock"
                    shape=unity7_canvas.get().draw_integer(
                        &ZAdic::new_approx(7, 20, vec![2, 4, 6, 3, 0, 2, 6, 2, 4, 3, 4, 4, 5, 2, 1, 2, 1, 4, 6, 1])
                    )
                />
                <ShapeCard class="clock"
                    shape=unity7_canvas.get().draw_integer(
                        &ZAdic::new_approx(7, 20, vec![3, 4, 6, 3, 0, 2, 6, 2, 4, 3, 4, 4, 5, 2, 1, 2, 1, 4, 6, 1])
                    )
                />
                <ShapeCard class="clock"
                    shape=unity7_canvas.get().draw_integer(
                        &ZAdic::new_approx(7, 20, vec![4, 2, 0, 3, 6, 4, 0, 4, 2, 3, 2, 2, 1, 4, 5, 4, 5, 2, 0, 5])
                    )
                />
                <ShapeCard class="clock"
                    shape=unity7_canvas.get().draw_integer(
                        &ZAdic::new_approx(7, 20, vec![5, 2, 0, 3, 6, 4, 0, 4, 2, 3, 2, 2, 1, 4, 5, 4, 5, 2, 0, 5])
                    )
                />
                <ShapeCard class="clock"
                    shape=unity7_canvas.get().draw_integer(
                        &ZAdic::new_approx(7, 20, vec![6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6])
                    )
                />
            </Flex>

        </Collapse>
    };

    let sqrt2_7powadic = view! {
        <Collapse title="7^n-adic \\sqrt{2}">

            <Flex>
                <ShapeCard class="clock"
                    title="7^1"
                    shape=ClockCanvas::builder().base(7).depth(20).build().draw_integer(
                        &EAdic::primed_from(7, 2).nth_root(2, 20).unwrap().into_roots().next().unwrap()
                    )
                />
                <ShapeCard class="clock"
                    title="7^2"
                    shape=ClockCanvas::builder().base(49).depth(20).build().draw_integer(
                        &PowAdic::new(EAdic::primed_from(7, 2).nth_root(2, 40).unwrap().into_roots().next().unwrap(), 2)
                    )
                />
                <ShapeCard class="clock"
                    title="7^3"
                    shape=ClockCanvas::builder().base(343).depth(20).build().draw_integer(
                        &PowAdic::new(EAdic::primed_from(7, 2).nth_root(2, 60).unwrap().into_roots().next().unwrap(), 3)
                    )
                />
                <ShapeCard class="clock"
                    title="7^4"
                    shape=ClockCanvas::builder().base(2401).depth(20).build().draw_integer(
                        &PowAdic::new(EAdic::primed_from(7, 2).nth_root(2, 80).unwrap().into_roots().next().unwrap(), 4)
                    )
                />
                <ShapeCard class="clock"
                    title="7^5"
                    shape=ClockCanvas::builder().base(16807).depth(20).build().draw_integer(
                        &PowAdic::new(EAdic::primed_from(7, 2).nth_root(2, 100).unwrap().into_roots().next().unwrap(), 5)
                    )
                />
            </Flex>

        </Collapse>
    };

    let composite_canvas = Signal::derive(
        move || ClockCanvas::builder().base(10).depth(20).build()
    );
    let composite_10adic = view! {
        <Collapse title="10-adic">

            <Flex>
                <ShapeCard class="clock"
                    title="10-adic simple"
                    shape=composite_canvas.get().draw_integer(&MAdic::approx_from_i32(10, 3, 20).unwrap())
                />
                <ShapeCard class="clock"
                    title="10-adic idempotent: 2^\\inf"
                    shape=composite_canvas.get().draw_integer(&MAdic::from_pure_p_adic(10, ZAdic::new_approx(5, 40, vec![1])).unwrap())
                />
                <ShapeCard class="clock"
                    title="10-adic idempotent: 5^\\inf"
                    shape=composite_canvas.get().draw_integer(&MAdic::from_pure_p_adic(10, ZAdic::new_approx(2, 100, vec![1])).unwrap())
                />
            </Flex>

        </Collapse>
    };

    let result_canvas = Signal::derive(
        move || ClockCanvas::builder().base(5).depth(6).show_val_circles(true).build()
    );
    let clock_result = view! {
        <Collapse title="Clock result component">

            <Flex>
                <ShapeCard class="clock debug-outline"
                    title="Ok clock"
                    shape=result_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-5, 24)).unwrap())
                />
                <ShapeCard<ClockShape> class="clock debug-outline"
                    title="Err clock"
                    shape=Err(AdicShapeError::ImproperConfig("Something didn't work".to_string()))
                />
            </Flex>

        </Collapse>
    };

    view! {
        <section class="boxed-section">
            {basic}
            {fractional}
            {component_vs_card}
            {sqrt2_7adic}
            {sqrt7_3adic}
            {roots_of_unity}
            {sqrt2_7powadic}
            {composite_10adic}
            {clock_result}
        </section>
    }

}
