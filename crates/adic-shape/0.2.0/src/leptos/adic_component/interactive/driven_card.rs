use adic::{
    normed::Valuation,
    traits::{AdicPrimitive, CanApproximate, HasDigits, HasApproximateDigits},
    QAdic, ZAdic,
};
use leptos::{either::EitherOf3, logging::log, prelude::*};
use thaw::{
    Body1, Card, CardFooter, CardHeader, CardPreview,
};
use crate::{
    error::AdicShapeResult,
    interactive::{InteractiveShapeOptions, ShapeControls, ShapeType},
    leptos::{util::mount_style, ShapeComponent},
    shape::{AdicCanvas, ClockCanvas, EuclideanCanvas, TreeCanvas},
};
use super::shape_controls::InteractiveShapeControls;



#[component]
/// Driven shape leptos card
///
/// ```no_run
/// # use adic::{traits::PrimedFrom, QAdic};
/// # use adic_shape::leptos::{DrivenShapeCard, InteractiveShapeCard};
/// # use leptos::prelude::*;
/// let input_num = signal(Ok(QAdic::primed_from(5, 0)));
/// let input_interactive_card = move || view! {
///     <InteractiveShapeCard title="Input"
///         adic_result_setter=input_num.1
///     />
/// };
/// let output_num = Signal::derive(move || Ok(-input_num.0.get()?));
/// let output_driven_card = move || view! {
///     <DrivenShapeCard title="Output"
///         adic_num=output_num
///     />
/// };
/// let both = view! {
///     {input_interactive_card}
///     {output_driven_card}
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn DrivenShapeCard(
    #[prop(into)]
    /// Provide the `ZAdic` number to the card
    adic_num: Signal<AdicShapeResult<QAdic<ZAdic>>>,
    #[prop(into, optional)]
    /// Interactive options
    options: RwSignal<InteractiveShapeOptions>,
    #[prop(into, optional)]
    /// Card HTML class
    class: Signal<Option<String>>,
    #[prop(into, optional)]
    /// Card title
    title: Signal<String>,
) -> impl IntoView {

    const MAX_DEPTH: isize = 100;

    if let Err(err) = mount_style(
        "shape-component",
        include_str!("../shape-component.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    // Set up controls
    let controls = RwSignal::new(ShapeControls {
        enable_depth_control: false,
        ..Default::default()
    });

    // Change depth based on adic_num input
    // TODO: Try to avoid the Effect, instead produce a derived controls object from the combination
    Effect::watch(
        move || adic_num.get(),
        move |num, _, _| {
            let depth = match num.as_ref().map(HasApproximateDigits::certainty) {
                Err(_) => MAX_DEPTH,
                Ok(Valuation::PosInf) => MAX_DEPTH,
                Ok(Valuation::Finite(v)) if v > MAX_DEPTH => MAX_DEPTH,
                Ok(Valuation::Finite(v)) => v,
            };
            controls.write().depth = depth;
        },
        true
    );


    // Interactive outer card
    let class = Signal::derive(move || [
        class.get().unwrap_or_default(),
        " shape-card".to_string()
    ].concat());
    view! {
        <Card class=class>

            <CardHeader class="shape-card-header">
                <h3 class="full-width center-text">{ move || title.get() }</h3>
            </CardHeader>

            <CardPreview class="shape-card-preview">
                <DrivenShapeComponent
                    adic_num
                    controls options
                />
            </CardPreview>

            <CardFooter class="shape-card-footer">
                <Body1 class="full-width">
                    <DrivenControls
                        adic_result=adic_num
                        controls options
                    />
                </Body1>
            </CardFooter>

        </Card>
    }

}


#[component]
fn DrivenShapeComponent(
    #[prop(into)]
    /// Provide the `ZAdic` number to the card
    adic_num: Signal<AdicShapeResult<QAdic<ZAdic>>>,
    #[prop(into)]
    /// Interactive controls, e.g. shape type
    controls: Signal<ShapeControls>,
    #[prop(into, optional)]
    /// Interactive options
    options: Signal<InteractiveShapeOptions>,
) -> impl IntoView {

    let either_view = move || match controls.get().shape_type {
        ShapeType::Clock => {
            let clock_shape = adic_num.get().and_then(|a| {
                let canvas = ClockCanvas::builder()
                    .base(a.base().into())
                    .depth(controls.get().depth)
                    .show_tick_labels(options.get().display_clock_numbers)
                    .build();
                canvas.draw_number(&a)
            });
            EitherOf3::A(view! {
                <ShapeComponent shape=clock_shape class="interactive-clock"/>
            })
        },
        ShapeType::Tree => {
            let direction = options.get().tree_direction;
            let dangling_direction = Some(direction.cwise());
            let tree_shape = adic_num.get().and_then(|a| {
                let canvas = TreeCanvas::builder()
                    .base(a.base().into())
                    .depth(controls.get().depth)
                    .direction(direction)
                    .dangling_direction(dangling_direction)
                    .build();
                canvas.draw_number(&a)
            });
            EitherOf3::B(view! {
                <ShapeComponent shape=tree_shape class="interactive-tree"/>
            })
        },
        ShapeType::Euclidean => {
            let scaling = options.get().euclidean_scale;
            let direction = options.get().euclidean_direction;
            let orientation = options.get().euclidean_orientation;
            let enclosing_disks = (0..options.get().euclidean_enclosing_disks).collect::<Vec<_>>();
            let euclidean_shape = adic_num.get().and_then(|a| {
                let canvas = EuclideanCanvas::builder()
                    .characteristic_p_adic(a.p())
                    .depth(controls.get().depth)
                    .scaling(scaling)
                    .direction(direction).orientation(orientation)
                    .resize_around_zero()
                    .draw_enclosing_disks(enclosing_disks)
                    .build();
                canvas.draw_number(&a)
            });
            EitherOf3::C(view! {
                <ShapeComponent shape=euclidean_shape class="interactive-euclidean"/>
            })
        },
    };

    view! {
        {move || either_view()}
    }

}


#[component]
fn DrivenControls(
    #[prop(into)]
    /// ZAdic number to display
    adic_result: Signal<AdicShapeResult<QAdic<ZAdic>>>,
    #[prop(into)]
    /// Controls
    controls: RwSignal<ShapeControls>,
    #[prop(into)]
    /// Options modified by this settings menu
    options: RwSignal<InteractiveShapeOptions>,
) -> impl IntoView {

    // Adic number display
    let adic_number_display = Signal::derive(move || {
        match adic_result.get() {
            Ok(a) => a.approximation(10).to_string(),
            Err(_) => "----------".to_string(),
        }
    });

    view! {
        <InteractiveShapeControls attr:class="interactive-2-menu" controls=controls options=options/>
        <table class="interactive-2-menu">

            <th>"Adic number"</th>

            // Display the string for the adic number
            <tr>
                <td class=TEXT_CLASS>{adic_number_display}</td>
            </tr>

        </table>
    }

}


const TEXT_CLASS: &str = "center-text";
