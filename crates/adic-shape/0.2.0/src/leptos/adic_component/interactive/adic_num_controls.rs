use adic::{
    traits::{CanApproximate, PrimedFrom},
    EAdic, QAdic, ZAdic,
};
use leptos::prelude::*;
use thaw::Select;
use crate::{
    error::AdicShapeResult,
    interactive::{AdicNumControls, AdicNumSource},
    leptos::SpinSlide,
};



#[component]
pub fn AdicNumControls(
    #[prop(into)]
    /// ZAdic number to display
    adic_result: Signal<AdicShapeResult<QAdic<ZAdic>>>,
    #[prop(into)]
    /// Controls
    controls: RwSignal<AdicNumControls>,
) -> impl IntoView {

    // Adic number display
    let adic_number_display = Signal::derive(move || {
        match adic_result.get() {
            Ok(a) => a.approximation(10).to_string(),
            Err(_) => "----------".to_string(),
        }
    });

    // Adic source selection
    let adic_source_select = (
        Signal::derive(move || controls.get().adic_source.to_string()),
        SignalSetter::<String>::map(move |new_val| {
            controls.write().adic_source = new_val.parse().expect("Unknown adic source option");
        })
    );
    let adic_source_options = view!{
        <option value="iadic">"Integer"</option>
        <option value="radic">"Rational"</option>
        <option value="preset">"Preset"</option>
    };

    // Prime selection
    let prime_select = (
        Signal::derive(move || controls.get().p.to_string()),
        SignalSetter::<String>::map(move |new_val| {
            controls.write().p = new_val.parse().expect("Unknown prime option");
        })
    );
    let prime_options = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31].map(|prime| view!{
        <option value=prime.to_string()>{prime.to_string()}</option>
    });
    let prime_select_disabled = move || controls.get().adic_source == AdicNumSource::Preset;

    view! {
        <table>

            <th>"Adic number"</th>

            // Display the string for the adic number
            <tr>
                <td class=TEXT_CLASS>{adic_number_display}</td>
            </tr>

            // Choose adic source
            <tr>
                <td><fieldset class="interactive-fieldset">
                    <legend>"Source"</legend>
                    <Select class=SELECT_CLASS name="adic-source"
                        value=adic_source_select default_value="iadic"
                    >
                        {adic_source_options}
                    </Select>
                </fieldset></td>
            </tr>

            // Choose prime
            <tr>
                <td><fieldset class="interactive-fieldset">
                    <legend>"Prime"</legend>
                    <Select
                        disabled=prime_select_disabled
                        class=SELECT_CLASS name="prime"
                        value=prime_select default_value="5"
                    >
                        {prime_options}
                    </Select>
                </fieldset></td>
            </tr>

            <IndividualAdicControls controls/>

        </table>
    }

}


#[component]
fn IndividualAdicControls(
    controls: RwSignal<AdicNumControls>,
) -> impl IntoView {

    const FIVE_TO_THIRTEEN: i32 = 1_220_703_125;
    let min_integer = -FIVE_TO_THIRTEEN;
    let min_integer_slider = -1000;
    let max_integer = FIVE_TO_THIRTEEN;
    let max_integer_slider = 1000;
    let min_numer = -FIVE_TO_THIRTEEN;
    let min_numer_slider = -1000;
    let max_numer = FIVE_TO_THIRTEEN;
    let max_numer_slider = 1000;
    let min_denom = 1;
    let min_denom_slider = 1;
    let max_denom = FIVE_TO_THIRTEEN.unsigned_abs();
    let max_denom_slider = 1000;

    // From integer
    let from_int_input = (
        Signal::derive(move || controls.get().from_int_val),
        SignalSetter::<i32>::map(move |new_val| controls.write().from_int_val = new_val)
    );

    // From rational (numerator)
    let from_rat_numer_input = (
        Signal::derive(move || controls.get().numer),
        SignalSetter::<i32>::map(move |new_val| controls.write().numer = new_val)
    );

    // From rational (denominator)
    let from_rat_denom_input = (
        Signal::derive(move || controls.get().denom),
        SignalSetter::<u32>::map(move |new_val| controls.write().denom = new_val)
    );

    // Preset selection
    let preset_idx_select = (
        Signal::derive(move || controls.get().preset_idx.to_string()),
        SignalSetter::<String>::map(move |new_val| {
            controls.write().preset_idx = new_val.parse().expect("Unknown preset option");
        })
    );
    let preset_options = PRESET_NAMES.into_iter().enumerate().map(|(idx, preset_name)| view!{
        <option value=idx.to_string()>{preset_name}</option>
    }).collect_view();

    view! {

        // Adic from integer
        <tr style=move || {
            if controls.get().adic_source == AdicNumSource::FromInteger { "display: table-row" } else { "display: none" }
        }>
            <td><fieldset class="interactive-fieldset">
                <legend>"Integer"</legend>
                <SpinSlide<i32>
                    class=INPUT_NUM_CLASS name="from-integer-input" value=from_int_input
                    step=1 min=min_integer max=max_integer
                    min_slider=min_integer_slider max_slider=max_integer_slider
                />
            </fieldset></td>
        </tr>

        // Adic from rational
        <tr style=move || {
            if controls.get().adic_source == AdicNumSource::FromRational { "display: table-row" } else { "display: none" }
        }>
            <td><fieldset class="interactive-fieldset">
                <legend>"Numerator"</legend>
                <SpinSlide<i32>
                    class=INPUT_NUM_CLASS name="from-rational-numer-input" value=from_rat_numer_input
                    step=1 min=min_numer max=max_numer
                    min_slider=min_numer_slider max_slider=max_numer_slider
                />
            </fieldset></td>
        </tr>
        <tr style=move || {
            if controls.get().adic_source == AdicNumSource::FromRational { "display: table-row" } else { "display: none" }
        }>
            <td><fieldset class="interactive-fieldset">
                <legend>"Denominator"</legend>
                <SpinSlide<u32>
                    class=INPUT_NUM_CLASS name="from-rational-denom-input" value=from_rat_denom_input
                    step=1 min=min_denom max=max_denom
                    min_slider=min_denom_slider max_slider=max_denom_slider
                />
            </fieldset></td>
        </tr>

        // Adic from preset
        <tr style=move || {
            if controls.get().adic_source == AdicNumSource::Preset { "display: table-row" } else { "display: none" }
        }>
            <td><fieldset class="interactive-fieldset">
                <legend>"Preset"</legend>
                <Select class=SELECT_CLASS name="preset"
                    value=preset_idx_select default_value="0"
                >
                    {preset_options}
                </Select>
            </fieldset></td>
        </tr>

    }

}


const TEXT_CLASS: &str = "center-text";
const SELECT_CLASS: &str = "interactive-select bright-boxed";
const INPUT_NUM_CLASS: &str = "interactive-input-num bright-boxed";

const PRESET_NAMES: [&str; 7] = [
    "7-adic 1st sqrt(2)",
    "7-adic 2nd sqrt(2)",
    "5-adic -1/4",
    "5-adic 1st fourthrt(1)",
    "5-adic 2nd fourthrt(1)",
    "5-adic 3rd fourthrt(1)",
    "5-adic 4th fourthrt(1)",
];

pub fn preset_num(idx: usize, depth: isize) -> AdicShapeResult<QAdic<ZAdic>> {
    let two_7_sqrts = QAdic::<ZAdic>::primed_from(7, 2).nth_root(2, depth)?.into_roots().collect::<Vec<_>>();
    let roots_of_unity_5 = QAdic::<ZAdic>::primed_from(5, 1).nth_root(4, depth)?.into_roots().collect::<Vec<_>>();
    let a = match idx {
        0 => two_7_sqrts[0].clone(),
        1 => two_7_sqrts[1].clone(),
        2 => QAdic::from_integer(EAdic::new_repeating(5, vec![], vec![1])).into_approximation(depth),
        3 => roots_of_unity_5[0].clone(),
        4 => roots_of_unity_5[1].clone(),
        5 => roots_of_unity_5[2].clone(),
        6 => roots_of_unity_5[3].clone(),
        _ => panic!("No preset with given idx")
    };
    Ok(a)
}
