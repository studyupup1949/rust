use leptos::{prelude::*, logging::log};
use crate::ClockShape;
use super::{mount_style, ComponentDisplay};


#[component]
/// Leptos component SVG for an adic clock
///
/// ```no_run
/// # use adic::radic;
/// # use adic_shape::{ClockComponent, ClockShape, ClockShapeOptions};
/// # use leptos::prelude::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let neg_one_fourth = radic!(5, [], [1]);
/// let num_digits = 6;
/// let adic_clock = ClockShape::new(&neg_one_fourth, num_digits, ClockShapeOptions::default())?;
/// let clock_view = view! {
///     <ClockComponent class="clock" clock_shape=adic_clock/>
/// };
/// # Ok(()) }
/// ```
pub fn ClockComponent(
    #[prop(into)]
    /// Clock shape to display in SVG
    clock_shape: Signal<ClockShape>,
    #[prop(into, optional)]
    /// HTML class
    class: Option<String>,
) -> impl IntoView {

    if let Err(err) = mount_style(
        "clock-component",
        include_str!("./clock-component.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    let comp_signal = move || {
        clock_shape.get().create_component(class.clone())
    };

    view! {
        {comp_signal}
    }

}


impl ComponentDisplay for ClockShape { }
