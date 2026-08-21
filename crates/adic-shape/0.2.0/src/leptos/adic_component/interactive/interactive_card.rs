use adic::{
    normed::{UltraNormed, Valuation},
    traits::{AdicPrimitive, CanApproximate, HasDigits, PrimedFrom},
    EAdic, QAdic, ZAdic,
};
use leptos::{either::EitherOf3, logging::log, prelude::*};
use thaw::{
    Body1, Card, CardFooter, CardHeader, CardPreview,
};
use crate::{
    error::{AdicShapeError, AdicShapeResult},
    interactive::{AdicNumControls, AdicNumSource, InteractiveShapeOptions, ShapeControls, ShapeType},
    leptos::{util::mount_style, ShapeComponent},
    shape::{AdicCanvas, ClockCanvas, EuclideanCanvas, TreeCanvas},
};
use super::{
    adic_num_controls::{preset_num, AdicNumControls},
    shape_controls::InteractiveShapeControls,
};



#[component]
/// Interactive shape leptos card
///
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::leptos::InteractiveShapeCard;
/// # use leptos::prelude::*;
/// let interactive_card = view! {
///     <InteractiveShapeCard class="interactive-shape"
///         title="Title message"
///     />
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn InteractiveShapeCard(
    #[prop(into, optional)]
    /// Retrieve the `ZAdic` number from the card
    adic_result_setter: Option<WriteSignal<AdicShapeResult<QAdic<ZAdic>>>>,
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

    if let Err(err) = mount_style(
        "shape-component",
        include_str!("../shape-component.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    // Set up controls
    let num_input_controls = RwSignal::new(AdicNumControls::default());
    let controls = RwSignal::new(ShapeControls::default());


    // Set up adic integer, and attach adic_result_setter
    // TODO: Use adic number here instead
    let adic_int = Signal::derive(move || {
        let AdicNumControls { p, adic_source, from_int_val, numer, denom, preset_idx } = num_input_controls.get();
        let ShapeControls { depth, .. } = controls.get();
        let a = match adic_source {
            AdicNumSource::FromInteger => {
                let a = QAdic::<EAdic>::primed_from(p, from_int_val);
                let a = a.into_approximation(depth);
                Ok(a)
            },
            AdicNumSource::FromRational => {
                let numer_a = QAdic::<EAdic>::primed_from(p, numer);
                let denom_a = QAdic::<EAdic>::primed_from(p, denom);
                let Valuation::Finite(dval) = denom_a.valuation() else {
                    return Err(AdicShapeError::Math("Cannot divide by zero".to_string()));
                };
                let approx_numer_a = numer_a.into_approximation(depth + dval);
                let a = approx_numer_a / denom_a.into();
                Ok(a)
            },
            AdicNumSource::Preset => {
                preset_num(preset_idx, depth)
            },
        };

        if let Some(setter) = adic_result_setter {
            setter.set(a.clone());
        }
        if let Ok(a) = a.as_ref() {
            let ap = a.p().into();
            if ap != p {
                num_input_controls.write().p = ap;
            }
        }

        a

    });


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
                <InteractiveShapeComponent
                    adic_result=adic_int
                    controls options
                />
            </CardPreview>

            <CardFooter class="shape-card-footer">
                <Body1 class="full-width">
                    <InteractiveControls
                        adic_result=adic_int
                        num_input_controls controls options
                    />
                </Body1>
            </CardFooter>

        </Card>
    }

}


#[component]
fn InteractiveShapeComponent(
    #[prop(into)]
    /// ZAdic number to display
    adic_result: Signal<AdicShapeResult<QAdic<ZAdic>>>,
    #[prop(into)]
    /// Interactive controls, e.g. shape type
    controls: Signal<ShapeControls>,
    #[prop(into, optional)]
    /// Interactive options
    options: Signal<InteractiveShapeOptions>,
) -> impl IntoView {

    let either_view = move || match controls.get().shape_type {
        ShapeType::Clock => {

            let clock_shape = adic_result.get().and_then(|a| {
                let canvas = ClockCanvas::builder()
                    .base(a.base().into())
                    .depth(controls.get().depth)
                    .show_tick_labels(options.get().display_clock_numbers)
                    .build();
                let shape = canvas.draw_number(&a);
                shape
            });

            EitherOf3::A(view! {
                <ShapeComponent shape=clock_shape class="interactive-clock"/>
            })

        },
        ShapeType::Tree => {
            let direction = options.get().tree_direction;
            let dangling_direction = Some(direction.cwise());
            let tree_shape = adic_result.get().and_then(|a| {
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
            let euclidean_shape = adic_result.get().and_then(|a| {
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
fn InteractiveControls(
    #[prop(into)]
    /// ZAdic number to display
    adic_result: Signal<AdicShapeResult<QAdic<ZAdic>>>,
    #[prop(into)]
    /// Adic number input controls
    num_input_controls: RwSignal<AdicNumControls>,
    #[prop(into)]
    /// Controls
    controls: RwSignal<ShapeControls>,
    #[prop(into)]
    /// Options modified by this settings menu
    options: RwSignal<InteractiveShapeOptions>,
) -> impl IntoView {

    view! {
        <InteractiveShapeControls attr:class="interactive-2-menu" controls=controls options=options/>
        <AdicNumControls attr:class="interactive-2-menu" adic_result=adic_result controls=num_input_controls/>
    }

}
