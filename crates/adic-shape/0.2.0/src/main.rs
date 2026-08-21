//! # Adic shape
//!
//! Use this cli to generate adic clock and tree svgs

use adic::{
    traits::{HasDigits, PrimedFrom},
    EAdic,
};
use adic_shape::{
    error::{AdicShapeError, AdicShapeResult},
    shape::{AdicCanvas, ClockCanvas, Direction, EuclideanCanvas, TreeCanvas},
    svg::SvgDisplay,
};
use clap::{ArgGroup, Parser, Subcommand};


/// Struct for command line
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Command being called
    #[command(subcommand)]
    pub command: CliCommands,
}

/// CLI command
#[derive(Debug, Subcommand)]
pub enum CliCommands {

    /// Generate a clock svg for an adic integer
    #[clap(group(
        ArgGroup::new("display_type")
            .required(true)
            .args(&["clock", "full_tree", "zoomed_tree", "euclidean"])
    ))]
    #[clap(group(
        ArgGroup::new("save_or_print")
            .required(true)
            .args(&["print", "save"])
    ))]
    GenerateSvg {

        #[arg(long)]
        /// Display adic integer as a clock
        clock: bool,

        #[arg(long)]
        /// Display adic integer as a full tree
        full_tree: bool,

        #[arg(long)]
        /// Display adic integer as a zoomed tree
        zoomed_tree: bool,

        #[arg(long)]
        /// Display adic integer as a full euclidean
        euclidean: bool,

        #[arg(short)]
        /// Prime for adic number and number of ticks on the clock
        p: u32,

        #[arg(short, allow_hyphen_values=true)]
        /// Integer the shape represents
        a: i32,

        #[arg(short, long, value_parser=clap::value_parser!(u32).range(1..26))]
        /// Depth of the shape
        depth: u32,

        #[arg(long)]
        /// Print the svg to terminal
        print: bool,

        #[arg(long)]
        /// Save the svg to the given file
        save: Option<String>,

    },

}


fn main() -> AdicShapeResult<()> {
    let args = Cli::parse();
    match args.command {

        CliCommands::GenerateSvg {clock, full_tree, zoomed_tree, euclidean, p, a, depth, print, save} => {

            let a = EAdic::primed_from(p, a);
            let isize_depth = depth.try_into()?;

            let svg_doc = if clock {

                // Create the clock
                let clock_canvas = ClockCanvas::builder().base(a.base().into()).depth(isize_depth).build();
                let clock_shape = clock_canvas.draw_integer(&a)?;
                clock_shape.create_svg_doc()

            } else if full_tree {

                if p.pow(depth) > 10000 {
                    Err(AdicShapeError::ImproperConfig(
                        "Cannot generate a full tree with such large depth; keep p^depth under 10000".to_string()
                    ))?;
                }

                // Create the full tree
                let tree_canvas = TreeCanvas::builder()
                    .base(p)
                    .depth(isize_depth)
                    .direction(Direction::Up)
                    .dangling_direction(Some(Direction::Down))
                    .solid_full_tree()
                    .build();
                let tree_shape = tree_canvas.draw_integer(&a)?;
                tree_shape.create_svg_doc()

            } else if zoomed_tree {

                // Create the zoomed tree
                let tree_canvas = TreeCanvas::builder()
                    .base(p)
                    .depth(isize_depth)
                    .direction(Direction::Up)
                    .dangling_direction(Some(Direction::Down))
                    .build();
                let tree_shape = tree_canvas.draw_integer(&a)?;
                tree_shape.create_svg_doc()

            } else if euclidean {

                if p.pow(depth) > 10000 {
                    Err(AdicShapeError::ImproperConfig(
                        "Cannot generate a full euclidean with such large depth; keep p^depth under 10000".to_string()
                    ))?;
                }

                // Create the euclidean with the characteristic p-adic mapping
                let scaling = 3.6;
                let euclidean_canvas = EuclideanCanvas::builder()
                    .characteristic_p_adic(p)
                    .scaling(scaling).depth(isize_depth)
                    .draw_scaled_hulls()
                    .solid_full_tree()
                    .build();
                let euclidean_shape = euclidean_canvas.draw_integer(&a)?;

                euclidean_shape.create_svg_doc()

            } else {
                return Err(AdicShapeError::ImproperConfig("Unknown CLI error".to_string()))
            };

            if print {
                println!("{svg_doc}");
            }
            if let Some(sf) = save {
                svg::save(sf, &svg_doc).unwrap();
            }

            Ok(())

        }

    }
}
