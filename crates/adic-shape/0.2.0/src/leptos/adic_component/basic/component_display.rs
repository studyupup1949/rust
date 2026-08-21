use itertools::Itertools;
use leptos::prelude::*;
use crate::{
    draw::{
        element::AdicEl,
        shape::{ClockShape, DisplayShape, EuclideanShape, TreeShape},
        util::str_digits,
    },
    error::AdicShapeResult,
};


/// For plotting different leptos components
///
/// This is probably not needed unless you are implementing a new shape.
/// You should instead use the display components directly,
///  e.g. [`ClockComponent`](crate::ClockComponent), [`TreeComponent`](crate::TreeComponent).
pub (crate) trait ComponentDisplay
where Self: DisplayShape {

    /// Leptos component for an adic shape
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


    /// Iterator through all the components of the svg shape
    fn shape_view(
        self,
    ) -> impl IntoView
    where Self: Sized {
        self.adic_els().map(|adic_el| match adic_el {
            AdicEl::Circle(c) => view! {
                <circle class={c.class}
                    cx={str_digits(c.cx, 5)} cy={str_digits(c.cy, 5)} r={str_digits(c.r, 5)}
                />
            }.into_any(),
            AdicEl::Path(p) => view! {
                <path class={p.class} d={p.d.into_iter().map(String::from).join(" ")}/>
            }.into_any(),
            AdicEl::Text(t) => view! {
                <text class={t.class} style={t.style}
                    x={str_digits(t.x, 5)} y={str_digits(t.y, 5)}
                    dx={str_digits(t.dx, 5)} dy={str_digits(t.dy, 5)}
                >
                    {t.content}
                </text>
            }.into_any(),
        }).collect_view()
    }

}



// IMPLS

impl From<ClockShape> for Signal<AdicShapeResult<ClockShape>> {
    fn from(value: ClockShape) -> Self {
        Signal::from(Ok(value))
    }
}
impl ComponentDisplay for ClockShape { }

impl From<EuclideanShape> for Signal<AdicShapeResult<EuclideanShape>> {
    fn from(value: EuclideanShape) -> Self {
        Signal::from(Ok(value))
    }
}
impl ComponentDisplay for EuclideanShape { }

impl From<TreeShape> for Signal<AdicShapeResult<TreeShape>> {
    fn from(value: TreeShape) -> Self {
        Signal::from(Ok(value))
    }
}
impl ComponentDisplay for TreeShape { }
