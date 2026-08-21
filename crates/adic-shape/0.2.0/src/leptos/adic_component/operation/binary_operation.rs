use adic::{traits::AdicPrimitive, QAdic};
use leptos::{logging::log, prelude::*};
use thaw::{Flex, FlexJustify, Select};

use crate::leptos::{
    util::mount_style,
    DrivenShapeCard, InteractiveShapeCard,
};
use super::{
    binary::BinaryOp,
    op_eq_arrow::OpEqArrow,
};



#[component]
/// Binary adic operation card
///
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::leptos::BinaryOpCard;
/// # use leptos::prelude::*;
/// let interactive_card = view! {
///     <BinaryOpCard/>
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn BinaryOpCard() -> impl IntoView {

    if let Err(err) = mount_style(
        "shape-component",
        include_str!("../shape-component.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    let input_num0 = signal(Ok(QAdic::zero(5)));
    let input_num1 = signal(Ok(QAdic::zero(5)));

    let op_type = RwSignal::new(BinaryOp::Add);

    let output_num = Signal::derive(
        move || op_type.get().call(input_num0.0.get()?, input_num1.0.get()?)
    );

    view! {
        <Flex justify=FlexJustify::Center>

            <InteractiveShapeCard class="clock"
                title="Num 1"
                adic_result_setter=input_num0.1
            />

            <BinaryAdicOp op_type=op_type/>

            <InteractiveShapeCard class="clock"
                title="Num 2"
                adic_result_setter=input_num1.1
            />

            <OpEqArrow/>

            <DrivenShapeCard class="clock"
                title="Output" adic_num=output_num
            />

        </Flex>
    }

}



#[component]
fn BinaryAdicOp(
    #[prop(into)]
    op_type: RwSignal<BinaryOp>,
) -> impl IntoView {

    let op_select = (
        Signal::derive(move || op_type.get().to_string()),
        SignalSetter::<String>::map(move |new_val| {
            op_type.set(new_val.parse().expect("Unknown operation option"));
        })
    );
    let op_type_options = [
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
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
                        value=op_select default_value="add"
                    >
                        {op_type_options}
                    </Select>
                </fieldset></td>
            </tr>

        </table>
    }

}
