use leptos::prelude::*;



#[component]
pub fn OpEqArrow() -> impl IntoView {

    view!{
        <svg class="eq-svg" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
            // <g stroke-width="0"></g>
            <g stroke-linecap="round" stroke-linejoin="round"></g>
            <g>
                <g>
                    <path d="M 0 30 L 80 30 M 0 70 L 80 70 M 70 20 L 100 50 L 70 80">
                    </path>
                </g>
            </g>
        </svg>
    }

}
