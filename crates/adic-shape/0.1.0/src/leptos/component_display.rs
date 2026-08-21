use itertools::Itertools;
use leptos::prelude::*;
use crate::{AdicEl, DisplayShape};


/// For plotting different leptos components
///
/// This is probably not needed unless you are implementing a new shape.
/// You should instead use the display components directly,
///  e.g. [`ClockComponent`](crate::ClockComponent), [`TreeComponent`](crate::TreeComponent).
pub trait ComponentDisplay: DisplayShape {

    /// Leptos component for an adic clock
    ///
    /// # Errors
    /// Throws an error if `adic_data` has infinite digits; you must supply an APPROXIMATE adic number.
    fn create_component(
        self,
        class: Option<String>,
    ) -> impl IntoView
    where Self: Sized {

        let class_str = match class {
            Some(c) => [self.default_class(), " ".to_string(), c].join(" "),
            None => self.default_class(),
        };
        let viewbox_str = self.viewbox_str();
        let comps = self.shape_view();

        view!{
            <svg class={class_str}
                viewBox={viewbox_str}
                xmlns="http://www.w3.org/2000/svg"
            >
                {comps}
            </svg>
        }

    }


    /// Iterator through all the components of the svg clock
    fn shape_view(
        self,
    ) -> impl IntoView
    where Self: Sized {
        self.adic_els().map(|adic_el| match adic_el {
            AdicEl::Circle(c) => view! {
                <circle class={c.class} cx={c.cx} cy={c.cy} r={c.r}/>
            }.into_any(),
            AdicEl::Path(p) => view! {
                <path class={p.class} d={p.d.into_iter().map(String::from).join(" ")}/>
            }.into_any(),
            AdicEl::Text(t) => view! {
                <text class={t.class} style={t.style}
                    x={t.x} y={t.y} dx={t.dx} dy={t.dy}
                >
                    {t.content}
                </text>
            }.into_any(),
        }).collect_view()
    }

}
