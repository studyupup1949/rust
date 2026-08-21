use adic::{traits::AdicPrimitive, QAdic};
use leptos::{logging::log, prelude::*};
use thaw::{Flex, FlexJustify, Select};

use crate::leptos::{
    util::mount_style,
    DrivenShapeCard, InteractiveShapeCard,
};
use super::{
    op_eq_arrow::OpEqArrow,
    unary::UnaryOp,
};



#[component]
/// Unary adic operation card
///
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::leptos::UnaryOpCard;
/// # use leptos::prelude::*;
/// let interactive_card = view! {
///     <UnaryOpCard/>
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn UnaryOpCard() -> impl IntoView {

    if let Err(err) = mount_style(
        "shape-component",
        include_str!("../shape-component.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    let input_num = signal(Ok(QAdic::zero(5)));

    let op_type = RwSignal::new(UnaryOp::Neg);

    let output_num = Signal::derive(
        move || op_type.get().call(input_num.0.get()?)
    );

    view! {
        <Flex justify=FlexJustify::Center>

            <InteractiveShapeCard class="clock"
                title="Input"
                adic_result_setter=input_num.1
            />

            <UnaryAdicOp op_type=op_type/>

            <OpEqArrow/>

            <DrivenShapeCard class="clock"
                title="Output"
                adic_num=output_num
            />

        </Flex>
    }

}



#[component]
fn UnaryAdicOp(
    #[prop(into)]
    op_type: RwSignal<UnaryOp>,
) -> impl IntoView {

    let op_select = (
        Signal::derive(move || op_type.get().to_string()),
        SignalSetter::<String>::map(move |new_val| {
            op_type.set(new_val.parse().expect("Unknown operation option"));
        })
    );
    let op_type_options = [
        UnaryOp::Neg,
        UnaryOp::Square,
        UnaryOp::Sqrt,
    ].map(|op_type| view!{
        <option value=op_type.to_string()>{op_type.to_string()}</option>
    });

    view! {
        <table class="op-table">

            <th>"Operation"</th>

            // Choose display shape type
            <tr>
                <td><fieldset class="operation-fieldset">
                    <legend>"Op"</legend>
                    <Select class="interactive-select bright-boxed" name="op-type"
                        value=op_select default_value="neg"
                    >
                        {op_type_options}
                    </Select>
                </fieldset></td>
            </tr>

        </table>
    }

}
