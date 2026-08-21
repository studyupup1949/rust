//! Leptos components for `adic-shape`

#![allow(unreachable_pub)]

mod adic_thaw;
mod adic_component;
mod util;

pub use adic_thaw::{Collapse, SpinSlide};
pub use adic_component::{
    AnimatedShapeCard, BinaryOpCard, DrivenShapeCard, InteractiveShapeCard,
    ShapeCard, ShapeComponent, UnaryOpCard,
};

#[cfg(feature="real_projection")]
#[cfg_attr(docsrs, doc(cfg(feature = "real_projection")))]
pub use adic_component::RealProjectionChart;
