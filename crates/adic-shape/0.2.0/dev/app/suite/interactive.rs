use leptos::prelude::*;

use adic_shape::leptos::InteractiveShapeCard;


#[component]
pub fn InteractiveSuite() -> impl IntoView {

    let basic = view! {
        <InteractiveShapeCard class="clock"
            title="Title message"
        />
    };

    view! {
        <section class="boxed-section">
            <h3>"Interactive examples"</h3>
            {basic}
        </section>
    }

}
