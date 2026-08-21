use std::{
    ops::{Add, Sub},
    str::FromStr,
};
use num::{Bounded, FromPrimitive, ToPrimitive};
use leptos::{either::Either, prelude::*};
use thaw::{
    Flex, Slider, SpinButton,
};
use thaw_utils::{BoxOneCallback, Model, OptionalProp};


#[component]
/// A `thaw` `SpinButton` and `Slider` grouped together and linked
///
/// Note: this component may be moved from this crate and not re-exported.
pub fn SpinSlide<T>(
    #[prop(optional, into)]
    class: MaybeProp<String>,
    /// A string specifying a name for the input control.
    /// This name is submitted along with the control's value when the form data is submitted.
    #[prop(optional, into)]
    name: MaybeProp<String>,
    /// Current value of the control.
    #[prop(optional, into)]
    value: Model<T>,
    /// Step amount for the `SpinButton`
    #[prop(into)]
    step: Signal<T>,
    /// The minimum number that the input value can take.
    #[prop(default = T::min_value().into(), into)]
    min: Signal<T>,
    /// The maximum number that the input value can take.
    #[prop(default = T::max_value().into(), into)]
    max: Signal<T>,
    /// Disable controls
    #[prop(default = false.into(), into)]
    disabled: Signal<bool>,
    /// Modifies the user input before assigning it to the value.
    #[prop(optional, into)]
    parser: OptionalProp<BoxOneCallback<String, Option<T>>>,
    /// Formats the value to be shown to the user.
    #[prop(optional, into)]
    format: OptionalProp<BoxOneCallback<T, String>>,
    /// The minimum number allowed by the slider, if different from the input minimum.
    #[prop(optional, into)]
    min_slider: MaybeProp<T>,
    /// The maximum number allowed by the slider, if different from the input maximum.
    #[prop(optional, into)]
    max_slider: MaybeProp<T>,
    /// Step amount for the slider, if different from the input step.
    #[prop(optional, into)]
    step_slider: MaybeProp<T>,
) -> impl IntoView
where
    T: Send + Sync,
    T: Add<Output = T> + Sub<Output = T> + PartialOrd + Bounded,
    T: ToPrimitive + FromPrimitive,
    T: Default + Clone + FromStr + ToString + 'static,
{

    let value_slider = (
        Signal::derive(move || value.get().to_f64().expect("value -> f64 conversion")),
        SignalSetter::<f64>::map(move |new_val| value.set(T::from_f64(new_val).expect("f64 -> value conversion")))
    );
    let min_slider_signal = Signal::derive(move || {
        let min = min_slider.get().unwrap_or(min.get());
        min.to_f64().expect("min -> f64 conversion")
    });
    let max_slider_signal = Signal::derive(move || {
        let max = max_slider.get().unwrap_or(max.get());
        max.to_f64().expect("max -> f64 conversion")
    });
    let step_slider_signal = Signal::derive(move || {
        let step = step_slider.get().unwrap_or(step.get());
        step.to_f64().expect("max -> f64 conversion")
    });

    let slider = move || if disabled.get() {
        Either::Left(())
    } else {
        Either::Right(view! {
            <Slider
                value=value_slider
                min=min_slider_signal max=max_slider_signal
                step=step_slider_signal show_stops=false
            />
        })
    };

    view! {
        <Flex class=class vertical=true>
            <SpinButton<T>
                name=name value=value
                step_page=step min=min max=max
                disabled=disabled
                parser=parser format=format
            />
            {slider}
        </Flex>
    }

}
