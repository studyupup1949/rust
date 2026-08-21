use leptos::{prelude::*, either::Either, logging::log};
use thaw::{Body1, Card, CardHeader, CardPreview};
use crate::{
    error::AdicShapeResult,
    leptos::{
        adic_component::basic::ComponentDisplay,
        util::mount_style,
    },
};



#[component]
/// Shape leptos SVG component
///
/// [`ShapeCard`] simply wraps this component in a `thaw` card.
/// Prefer that if looking simply for ease of use.
///
/// Clocks:
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::{leptos::ShapeComponent, shape::{AdicCanvas, ClockCanvas}};
/// # use leptos::prelude::*;
/// let p = 5;
/// let neg_one_fourth = EAdic::new_repeating(p, vec![], vec![1]);
/// let num_digits = 6;
///
/// let canvas = ClockCanvas::builder().base(p).depth(num_digits).build();
/// let clock_shape = canvas.draw_integer(&neg_one_fourth)?;
/// let clock_view = view! {
///     <ShapeComponent shape=clock_shape/>
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Trees:
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::{leptos::ShapeComponent, shape::{AdicCanvas, TreeCanvas}};
/// # use leptos::prelude::*;
/// let p = 5;
/// let neg_one_fourth = EAdic::new_repeating(p, vec![], vec![1]);
/// let num_digits = 6;
///
/// let canvas = TreeCanvas::builder().base(p).depth(num_digits).build();
/// let tree_shape = canvas.draw_integer(&neg_one_fourth)?;
/// let tree_view = view! {
///     <ShapeComponent shape=tree_shape/>
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Euclideans:
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::{leptos::ShapeComponent, shape::{AdicCanvas, EuclideanCanvas}};
/// # use leptos::prelude::*;
/// let p = 5;
/// let scaling = 1.5;
/// let depth = 10;
/// let neg_one_fourth = EAdic::new_repeating(p, vec![], vec![1]);
///
/// let canvas = EuclideanCanvas::builder()
///     .characteristic_p_adic(p)
///     .depth(depth)
///     .scaling(scaling)
///     .build();
/// let euclidean_shape = canvas.draw_integer(&neg_one_fourth)?;
/// let euclidean_view = view! {
///     <ShapeComponent shape=euclidean_shape/>
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn ShapeComponent<S>(
    #[prop(into)]
    /// Adic shape to display in SVG
    shape: Signal<AdicShapeResult<S>>,
    #[prop(into, optional)]
    /// Shape HTML class
    class: Signal<Option<String>>,
) -> impl IntoView
where S: Clone + Send + Sync + ComponentDisplay + 'static {

    if let Err(err) = mount_style(
        "shape-component",
        include_str!("../shape-component.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    let comp_signal = move || {
        match shape.get() {
            Ok(shape) => Either::Left(view!{
                {shape.create_component(class.get())}
            }),
            Err(err) => Either::Right(view! {
                <p class=class>{format!("Error while building shape: {err}")}</p>
            }),
        }
    };

    view! {
        {comp_signal}
    }

}


#[component]
/// Shape leptos card
///
/// This component simply wraps [`ShapeComponent`] in a `thaw` card.
/// Prefer this if looking simply for ease of use.
///
/// Clocks:
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::{leptos::ShapeCard, shape::{AdicCanvas, ClockCanvas}};
/// # use leptos::prelude::*;
/// let p = 5;
/// let neg_one_fourth = EAdic::new_repeating(p, vec![], vec![1]);
/// let num_digits = 6;
///
/// let canvas = ClockCanvas::builder().base(p).depth(num_digits).build();
/// let clock_shape = canvas.draw_integer(&neg_one_fourth)?;
/// let clock_view = view! {
///     <ShapeCard shape=clock_shape/>
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Trees:
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::{leptos::ShapeCard, shape::{AdicCanvas, TreeCanvas}};
/// # use leptos::prelude::*;
/// let p = 5;
/// let neg_one_fourth = EAdic::new_repeating(p, vec![], vec![1]);
/// let num_digits = 6;
///
/// let canvas = TreeCanvas::builder().base(p).depth(num_digits).build();
/// let tree_shape = canvas.draw_integer(&neg_one_fourth)?;
/// let tree_view = view! {
///     <ShapeCard shape=tree_shape/>
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Euclideans:
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::{leptos::ShapeCard, shape::{AdicCanvas, EuclideanCanvas}};
/// # use leptos::prelude::*;
/// let p = 5;
/// let scaling = 1.5;
/// let depth = 10;
///
/// let canvas = EuclideanCanvas::builder()
///     .characteristic_p_adic(p)
///     .depth(depth)
///     .scaling(scaling)
///     .build();
/// let neg_one_fourth = EAdic::new_repeating(p, vec![], vec![1]);
/// let euclidean_shape = canvas.draw_integer(&neg_one_fourth)?;
/// let euclidean_view = view! {
///     <ShapeCard shape=euclidean_shape/>
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn ShapeCard<S>(
    #[prop(into)]
    /// Adic shape to display in SVG
    shape: Signal<AdicShapeResult<S>>,
    #[prop(into, optional)]
    /// Card HTML class
    class: Option<String>,
    #[prop(into, optional)]
    /// Card title
    title: String,
) -> impl IntoView
where S: Clone + Send + Sync + ComponentDisplay + 'static {

    view! {
        <Card class=class>

            <CardHeader class="shape-card-header">
                <Body1>
                    <h3>{title}</h3>
                </Body1>
            </CardHeader>

            <CardPreview class="shape-card-preview">
                <ShapeComponent shape=shape/>
            </CardPreview>

        </Card>
    }

}
