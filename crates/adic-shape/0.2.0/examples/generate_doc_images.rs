//! Generate images used for `cargo doc`

use std::path::PathBuf;
use adic::{
    traits::{AdicInteger, PrimedFrom},
    EAdic, ZAdic,
};
use adic_shape::{
    shape::{AdicCanvas, ClockCanvas, EuclideanCanvas, TreeCanvas},
    svg::SvgDisplay,
};
use clap::Parser;


/// Struct for command line
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Directory to save the images
    #[arg(short, long)]
    save_dir: PathBuf,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Cli::parse();
    let save_dir = args.save_dir;


    // LIB EXAMPLES

    let two = EAdic::primed_from(7, 2);
    let adics = two.nth_root(2, 25)?.into_roots().collect::<Vec<_>>();

    // Top clock
    let depth = 25;
    let clock_canvas = ClockCanvas::builder()
        .base(7).depth(depth)
        .build();
    let shape = clock_canvas.draw_integer(&adics[0])?;
    let svg_doc = shape.create_svg_doc();
    svg::save(save_dir.join("clock-7-sqrt-2.svg"), &svg_doc)?;

    // Top tree
    let depth = 10;
    let tree_canvas = TreeCanvas::builder()
        .base(7).depth(depth)
        .build();
    let shape = tree_canvas.draw_integers(&adics)?;
    let svg_doc = shape.create_svg_doc();
    svg::save(save_dir.join("tree-7-sqrt-2.svg"), &svg_doc)?;

    // Top euclidean
    let depth = 3;
    let euclidean_canvas = EuclideanCanvas::builder()
        .characteristic_p_adic(7)
        .scaling(3.25).depth(depth)
        .draw_scaled_hulls()
        .solid_full_tree()
        .build();
    let shape = euclidean_canvas.draw_integers(&adics)?;
    let svg_doc = shape.create_svg_doc();
    svg::save(save_dir.join("euclidean-7-sqrt-2.svg"), &svg_doc)?;

    // Clock 158
    let depth = 6;
    let clock_canvas = ClockCanvas::builder()
        .base(5).depth(depth)
        .show_val_circles(true)
        .build();
    let shape = clock_canvas.draw_integer(&EAdic::primed_from(5, 158))?;
    let svg_doc = shape.create_svg_doc();
    svg::save(save_dir.join("clock-158.svg"), &svg_doc)?;

    // Full tree 158
    let depth = 3;
    let tree_canvas = TreeCanvas::builder()
        .base(5).depth(depth)
        .solid_full_tree()
        .build();
    let shape = tree_canvas.draw_integer(&EAdic::primed_from(5, 158))?;
    let svg_doc = shape.create_svg_doc();
    svg::save(save_dir.join("full-tree-158.svg"), &svg_doc)?;

    // Zoomed tree 158
    let depth = 5;
    let tree_canvas = TreeCanvas::builder()
        .base(5).depth(depth)
        .twig_depth(2)
        .build();
    let shape = tree_canvas.draw_integer(&EAdic::primed_from(5, 158))?;
    let svg_doc = shape.create_svg_doc();
    svg::save(save_dir.join("zoomed-tree-158.svg"), &svg_doc)?;

    // Euclidean 158
    let depth = 4;
    let euclidean_canvas = EuclideanCanvas::builder()
        .characteristic_p_adic(5)
        .scaling(2.8).depth(depth)
        .solid_full_tree()
        .draw_scaled_hulls()
        .build();
    let shape = euclidean_canvas.draw_integer(&EAdic::primed_from(5, 158))?;
    let svg_doc = shape.create_svg_doc();
    svg::save(save_dir.join("full-euclidean-158.svg"), &svg_doc)?;

    // Full euclidean roots of unity
    let depth = 4;
    let euclidean_canvas = EuclideanCanvas::builder()
        .characteristic_p_adic(5)
        .scaling(2.8).depth(depth)
        .solid_full_tree()
        .draw_scaled_hulls()
        .build();
    let unity_variety = ZAdic::roots_of_unity(5, 4)?;
    let roots = unity_variety.roots();
    let shape = euclidean_canvas.draw_integers(roots)?;
    let svg_doc = shape.create_svg_doc();
    svg::save(save_dir.join("full-euclidean-roots-of-unity.svg"), &svg_doc)?;

    // Simple euclidean roots of unity
    let depth = 4;
    let euclidean_canvas = EuclideanCanvas::builder()
        .characteristic_p_adic(5)
        .scaling(2.8).depth(depth)
        .build();
    let unity_variety = ZAdic::roots_of_unity(5, 4)?;
    let roots = unity_variety.roots();
    let shape = euclidean_canvas.draw_integers(roots)?;
    let svg_doc = shape.create_svg_doc();
    svg::save(save_dir.join("euclidean-roots-of-unity.svg"), &svg_doc)?;

    Ok(())

}
