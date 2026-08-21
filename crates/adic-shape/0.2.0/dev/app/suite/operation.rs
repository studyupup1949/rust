use leptos::prelude::*;
use adic_shape::leptos::{BinaryOpCard, Collapse, UnaryOpCard};


#[component]
pub fn OperationSuite() -> impl IntoView {

    view! {
        <section class="boxed-section">

            <Collapse title="Unary">
                <UnaryOpCard/>
            </Collapse>

            <Collapse title="Binary">
                <BinaryOpCard/>
            </Collapse>

        </section>
    }

}
