mod prototype;
mod suite;
mod theme;

use leptos::prelude::*;
use leptos_router::{
    components::{A, Router, Route, Routes},
    path,
};
use thaw::{Button, ButtonShape, Flex, Theme};
use adic_shape::leptos::Collapse;

use theme::{default_theme, ConfigProvider};


/// App entry point, hosts meta, router, fallback, and routes
#[component]
pub fn App() -> impl IntoView {

    let theme_signal = RwSignal::new(default_theme());

    view! {
        <Router>
            <ConfigProvider class="top-config" theme=theme_signal>
                <main>
                    <A href="/"><h1>"adic-shape"</h1></A>
                    <Routes fallback=|| "Not found.">

                        <Route path=path!("/") view=move || view! { <Home theme=theme_signal/> } />
                        <Route path=path!("/clock") view=suite::ClockSuite/>
                        <Route path=path!("/tree") view=suite::TreeSuite/>
                        <Route path=path!("/euclidean") view=suite::EuclideanSuite/>
                        <Route path=path!("/comparison") view=suite::ComparisonSuite/>
                        <Route path=path!("/interactive") view=suite::InteractiveSuite/>
                        <Route path=path!("/real-projection") view=suite::RealProjectionChartSuite/>
                        <Route path=path!("/animate") view=suite::AnimateSuite/>
                        <Route path=path!("/hensel") view=suite::HenselSuite/>
                        <Route path=path!("/operation") view=suite::OperationSuite/>
                        <Route path=path!("/thaw") view=suite::ThawSuite/>

                    </Routes>
                </main>
            </ConfigProvider>
        </Router>
    }

}


#[component]
fn Home(
    #[prop(into)]
    theme: RwSignal<Theme>,
) -> impl IntoView {
    view! {

        <Collapse title="Prototype">
            <prototype::PrototypeSuite/>
        </Collapse>

        <Collapse title="Theme">
            <theme::ThemeControl global_theme=theme/>
        </Collapse>

        <h2>"Test suites"</h2>
        <Flex>
            <A href="/clock"><Button shape=ButtonShape::Circular>"Clocks"</Button></A>
            <A href="/tree"><Button shape=ButtonShape::Circular>"Trees"</Button></A>
            <A href="/euclidean"><Button shape=ButtonShape::Circular>"Euclidean"</Button></A>
            <A href="/comparison"><Button shape=ButtonShape::Circular>"Shape Comparison"</Button></A>
            <A href="/interactive"><Button shape=ButtonShape::Circular>"Interactive"</Button></A>
            <A href="/real-projection"><Button shape=ButtonShape::Circular>"Real Projection Chart"</Button></A>
            <A href="/animate"><Button shape=ButtonShape::Circular>"Animation"</Button></A>
            <A href="/hensel"><Button shape=ButtonShape::Circular>"Hensel Examples"</Button></A>
            <A href="/operation"><Button shape=ButtonShape::Circular>"Operations"</Button></A>
            <A href="/thaw"><Button shape=ButtonShape::Circular>"Thaw Examples"</Button></A>
        </Flex>

    }
}
