use leptos::prelude::*;
use thaw::{
    Accordion, AccordionHeader, AccordionItem, Body1, Button, ButtonAppearance,
    Card, CardFooter, CardHeader, CardHeaderAction, CardPreview, Flex,
};

use adic_shape::leptos::Collapse;


#[component]
pub fn ThawSuite() -> impl IntoView {

    let basic = view! {
        <Collapse title="Basic">

            <h3>"Basic thaw components"</h3>

            <Flex>
                <Button appearance=ButtonAppearance::Primary>
                    "Primary"
                </Button>
                <Button appearance=ButtonAppearance::Secondary>
                    "Secondary"
                </Button>
                <Button appearance=ButtonAppearance::Subtle>
                    "Subtle"
                </Button>
                <Button appearance=ButtonAppearance::Transparent>
                    "Transparent"
                </Button>
            </Flex>

            <Card>

                <CardHeader>
                    <Body1>
                        <h3>"Header for Card"</h3>
                    </Body1>
                    // <CardHeaderDescription slot>
                    //     <Caption1>"Description for Card"</Caption1>
                    // </CardHeaderDescription>
                    <CardHeaderAction slot>
                        <Button appearance=ButtonAppearance::Transparent icon=icondata_fi::FiMoreVertical />
                    </CardHeaderAction>
                </CardHeader>

                <CardPreview>
                    <p>"Inside Card"</p>
                </CardPreview>

                <CardFooter>
                    <Button>"Reply"</Button>
                    <Button>"Share"</Button>
                </CardFooter>

            </Card>

            <Accordion multiple=true>
                <AccordionItem value="leptos">
                    <AccordionHeader slot>
                        "Leptos"
                    </AccordionHeader>
                    "Build fast web applications with Rust."
                </AccordionItem>
                <AccordionItem  value="thaw">
                    <AccordionHeader slot>
                        "Thaw"
                    </AccordionHeader>
                    "An easy to use leptos component library"
                </AccordionItem>
            </Accordion>

        </Collapse>
    };

    view! {
        <section class="boxed-section">
            {basic}
        </section>
    }

}
