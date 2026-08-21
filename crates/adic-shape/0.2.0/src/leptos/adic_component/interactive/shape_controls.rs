use leptos::prelude::*;
use thaw::{
    Select, SpinButton, Switch,
};
use crate::{
    interactive::{InteractiveShapeOptions, ShapeControls, ShapeType},
    leptos::SpinSlide,
};



#[component]
pub fn InteractiveShapeControls(
    #[prop(into)]
    /// Interactive controls, e.g. play/pause and time between ticks
    controls: RwSignal<ShapeControls>,
    #[prop(into)]
    /// Options modified by this settings menu
    options: RwSignal<InteractiveShapeOptions>,
) -> impl IntoView {

    // Conditions
    let when_clock = move || controls.get().shape_type == ShapeType::Clock;
    let when_tree = move || controls.get().shape_type == ShapeType::Tree;
    let when_euclidean = move || controls.get().shape_type == ShapeType::Euclidean;

    let individual_shape_controls = view! {

        <Show when=when_clock>
            <ClockSettings options/>
        </Show>

        <Show when=when_tree>
            <TreeSettings options/>
        </Show>

        <Show when=when_euclidean>
            <EuclideanSettings options/>
        </Show>

    };

    view! {
        <table>
            <FixedShapeControls controls/>
            <tr class="separator"><td/></tr>
            {individual_shape_controls}
        </table>
    }

}


#[component]
fn FixedShapeControls(
    controls: RwSignal<ShapeControls>,
) -> impl IntoView {

    let min_depth = 1;
    let max_depth = 100;

    // Shape type selection
    let shape_type_select = (
        Signal::derive(move || controls.get().shape_type.to_string()),
        SignalSetter::<String>::map(move |new_val| {
            controls.write().shape_type = new_val.parse().expect("Unknown shape type option");
        })
    );
    let shape_type_options = [
        ShapeType::Clock,
        ShapeType::Tree,
        ShapeType::Euclidean,
    ].map(|shape_type| view!{
        <option value=shape_type.to_string()>{shape_type.to_string()}</option>
    });

    // Depth
    let depth_input = (
        Signal::derive(move || controls.get().depth),
        SignalSetter::<isize>::map(move |new_val| controls.write().depth = new_val)
    );
    let depth_input_disabled = move || !controls.get().enable_depth_control;

    view!{

        <th>"Shape Controls"</th>

        // Choose display shape type
        <tr>
            <td><fieldset class="interactive-fieldset">
                <legend>"Shape"</legend>
                <Select class=SELECT_CLASS name="shape-type"
                    value=shape_type_select default_value="Clock"
                >
                    {shape_type_options}
                </Select>
            </fieldset></td>
        </tr>

        // Choose depth (or number of clock hands)
        <tr>
            <td><fieldset class="interactive-fieldset">
                <legend>"Depth"</legend>
                <SpinSlide<isize>
                    class=INPUT_NUM_CLASS name="depth-input"
                    disabled=depth_input_disabled
                    value=depth_input
                    step=1 min=min_depth max=max_depth
                />
            </fieldset></td>
        </tr>

    }

}

#[component]
fn ClockSettings(
    options: RwSignal<InteractiveShapeOptions>,
) -> impl IntoView {

    // Display clock numbers
    let display_clock_numbers_signal = (
        Signal::derive(move || options.get().display_clock_numbers),
        SignalSetter::map(move |new_val| options.write().display_clock_numbers = new_val)
    );
    view!{

        <th>"Clock settings"</th>

        <tr>
            <td><fieldset class="interactive-fieldset">
                <legend>"Display numbers"</legend>
                <Switch checked=display_clock_numbers_signal name="Display numbers"/>
            </fieldset></td>
        </tr>

    }

}

#[component]
fn TreeSettings(
    options: RwSignal<InteractiveShapeOptions>,
) -> impl IntoView {

    // Tree direction selection
    let tree_direction_getter = Signal::derive(move || options.get().tree_direction.to_string());
    let tree_direction_setter = SignalSetter::<String>::map(move |new_val| {
        options.write().tree_direction = new_val.parse().expect("Unknown tree direction option");
    });
    let tree_direction_select = (tree_direction_getter, tree_direction_setter);

    let direction_options = move || view! {
        <option value="Up">"Up"</option>
        <option value="Down">"Down"</option>
        <option value="Left">"Left"</option>
        <option value="Right">"Right"</option>
    };

    view! {

        <th>"Tree settings"</th>

        <tr>
            <td><fieldset class="interactive-fieldset">
                <legend>"Direction"</legend>
                <Select class=SELECT_CLASS name="tree-direction"
                    value=tree_direction_select default_value="Up"
                >
                    {direction_options}
                </Select>
            </fieldset></td>
        </tr>

    }

}

#[component]
fn EuclideanSettings(
    options: RwSignal<InteractiveShapeOptions>,
) -> impl IntoView {

    // Euclidean scale
    let euclidean_scale = (
        Signal::derive(move || options.get().euclidean_scale),
        SignalSetter::map(move |new_val| options.write().euclidean_scale = new_val)
    );
    let step_scale = 0.01;
    let min_scale = 1.0001;
    let max_scale = 5.0;
    let scale_input_format = move |v: f64| {
        if v > 1.005 {
            format!("{v:.2}")
        } else {
            format!("{v:.4}")
        }
    };

    // Euclidean direction selection
    let euclidean_direction_getter = Signal::derive(move || options.get().euclidean_direction.to_string());
    let euclidean_direction_setter = SignalSetter::<String>::map(move |new_val| {
        options.write().euclidean_direction = new_val.parse().expect("Unknown euclidean direction option");
    });
    let euclidean_direction_select = (euclidean_direction_getter, euclidean_direction_setter);

    // Euclidean orientation selection
    let euclidean_orientation_getter = Signal::derive(move || options.get().euclidean_orientation.to_string());
    let euclidean_orientation_setter = SignalSetter::<String>::map(move |new_val| {
        options.write().euclidean_orientation = new_val.parse().expect("Unknown euclidean orientation option");
    });
    let euclidean_orientation_select = (euclidean_orientation_getter, euclidean_orientation_setter);

    // Euclidean enclosing disks
    let euclidean_enclosing_disks_input = (
        Signal::derive(move || options.get().euclidean_enclosing_disks),
        SignalSetter::map(move |new_val| options.write().euclidean_enclosing_disks = new_val)
    );
    let min_disks = 0;
    let max_disks = 5;

    let direction_options = move || view! {
        <option value="Up">"Up"</option>
        <option value="Down">"Down"</option>
        <option value="Left">"Left"</option>
        <option value="Right">"Right"</option>
    };
    let orientation_options = move || view! {
        <option value="CW">"Clockwise"</option>
        <option value="CCW">"Counterclockwise"</option>
    };

    view! {

        <th>"Euclidean settings"</th>

        <tr>
            <td><fieldset class="interactive-fieldset">
                <legend>"Scaling"</legend>
                <SpinSlide<f64>
                    class=INPUT_NUM_CLASS name="euclidean-scale-input"
                    value=euclidean_scale
                    min=min_scale max=max_scale
                    step=step_scale format=scale_input_format
                    step_slider=step_scale
                />
            </fieldset></td>
        </tr>

        <tr>
            <td><fieldset class="interactive-fieldset">
                <legend>"Direction"</legend>
                <Select class=SELECT_CLASS name="euclidean-direction"
                    value=euclidean_direction_select default_value="Up"
                >
                    {direction_options}
                </Select>
            </fieldset></td>
        </tr>

        <tr>
            <td><fieldset class="interactive-fieldset">
                <legend>"Orientation"</legend>
                <Select class=SELECT_CLASS name="euclidean-orientation"
                    value=euclidean_orientation_select default_value="CW"
                >
                    {orientation_options}
                </Select>
            </fieldset></td>
        </tr>

        <tr>
            <td><fieldset class="interactive-fieldset">
                <legend>"Enclosing disks"</legend>
                <SpinButton<isize>
                    class=INPUT_NUM_CLASS name="enclosing-disks-input" value=euclidean_enclosing_disks_input
                    step_page=1 min=min_disks max=max_disks
                />
            </fieldset></td>
        </tr>

    }

}


const SELECT_CLASS: &str = "interactive-select bright-boxed";
const INPUT_NUM_CLASS: &str = "interactive-input-num bright-boxed";
