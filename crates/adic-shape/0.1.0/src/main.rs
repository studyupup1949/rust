//! # Adic shape
//!
//! Use this cli to generate adic clock and tree svgs

use adic::{IAdic, SignedAdicInteger};
use adic_shape::{
    AdicShapeError,
    ClockShape, ClockShapeOptions,
    Direction,
    SvgDocDisplay,
    TreeShape, TreeShapeOptions,
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
            .args(&["clock", "full_tree", "zoomed_tree"])
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

        #[arg(short)]
        /// Prime for adic number and number of ticks on the clock
        p: u32,

        #[arg(short, allow_hyphen_values=true)]
        /// Integer the clock represents
        a: i32,

        #[arg(short, long, value_parser=clap::value_parser!(u32).range(1..26))]
        /// Depth of the tree
        depth: u32,

        #[arg(long)]
        /// Print the svg to terminal
        print: bool,

        #[arg(long)]
        /// Save the svg to the given file
        save: Option<String>,

    },

}


fn main() -> Result<(), AdicShapeError> {
    let args = Cli::parse();
    match args.command {

        CliCommands::GenerateSvg {clock, full_tree, zoomed_tree, p, a, depth, print, save} => {

            let a = IAdic::from_i32(p, a);
            let usize_depth = depth.try_into()?;

            let svg_doc = if clock {

                // Create the clock
                let shape_options = ClockShapeOptions::default();
                let clock_shape = ClockShape::new(&a, usize_depth, shape_options)?;
                clock_shape.create_svg_doc()

            } else if full_tree {

                if p.pow(depth) > 1000 {
                    Err(AdicShapeError::ImproperConfig(
                        "Cannot generate a full tree with such large depth; keep p^depth under 1000".to_string()
                    ))?;
                }

                // Create the full tree
                let shape_options = TreeShapeOptions {
                    direction: Direction::Up,
                    dangling_direction: Some(Direction::Down),
                    ..Default::default()
                };
                let tree_shape = TreeShape::adic_number_full_tree(&a, usize_depth, shape_options)?;
                tree_shape.create_svg_doc()

            } else if zoomed_tree {

                // Create the zoomed tree
                let shape_options = TreeShapeOptions {
                    direction: Direction::Up,
                    dangling_direction: Some(Direction::Down),
                    ..Default::default()
                };
                let tree_shape = TreeShape::zoomed_tree(&a, usize_depth, shape_options)?;
                tree_shape.create_svg_doc()

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
