use leptos::{prelude::*, logging::log};
use crate::TreeShape;
use super::{mount_style, ComponentDisplay};
// use super::tree_labeller::TreeLabeller;


#[component]
/// Leptos component SVG for an adic tree
///
/// ```no_run
/// # use adic::radic;
/// # use adic_shape::{TreeComponent, TreeShape, TreeShapeOptions};
/// # use leptos::prelude::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let neg_one_fourth = radic!(5, [], [1]);
/// let num_digits = 6;
/// let adic_tree = TreeShape::zoomed_tree(&neg_one_fourth, num_digits, TreeShapeOptions::default())?;
/// let tree_view = view! {
///     <TreeComponent class="tree" tree_shape=adic_tree/>
/// };
/// # Ok(()) }
/// ```
pub fn TreeComponent(
    #[prop(into)]
    /// Tree shape to display in SVG
    tree_shape: TreeShape,
    #[prop(into, optional)]
    /// HTML class
    class: Option<String>,
    // #[prop(into, default=TreeLabeller::NoLabel)]
    // labeller: TreeLabeller,
) -> impl IntoView {

    if let Err(err) = mount_style(
        "tree-component",
        include_str!("./tree-component.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    tree_shape.create_component(class)

}


impl ComponentDisplay for TreeShape { }
