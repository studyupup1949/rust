use leptos::{prelude::*, logging::log};
use thaw::{Button, ButtonAppearance, Card, CardHeader, CardPreview, Flex};
use crate::leptos::util::mount_style;


#[component]
/// Collapsable element with togglable div/button header
///
/// Note: this component may be moved from this crate and not re-exported.
pub fn Collapse(
    children: Children,
    #[prop(into, default=None.into())]
    title: Signal<Option<String>>,
    #[prop(default=false)]
    start_open: bool,
) -> impl IntoView {

    if let Err(err) = mount_style(
        "collapse",
        include_str!("./collapse.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    let (collapsed, set_collapsed) = signal(!start_open);
    let on_button_click = move |_| { set_collapsed.set(!collapsed.get()) };
    let content_display = move || if collapsed.get() { "none" } else { "block" };
    let pm = move || if collapsed.get() { "+" } else { "-" };

    view!{

        <Card class="collapse-card">

            <CardHeader>
                <Flex class="collapse-header-flex">
                    <Button class="collapse-header-button" appearance=ButtonAppearance::Transparent block=true
                        on_click=on_button_click
                    >
                        <h2>{title}</h2>
                    </Button>
                    <h2 class="collapse-header-pm">{pm}</h2>
                </Flex>
            </CardHeader>

            <CardPreview class="collapse-content" style:display=content_display>
                {children()}
            </CardPreview>

        </Card>

    }

}
