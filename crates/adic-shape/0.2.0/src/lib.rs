// Would like to pass this in config rustdocflags, but relative paths don't work for some reason
#![doc = "<style>"]
#![doc = include_str!("../img/rustdoc.css")]
#![doc = "</style>"]
#![doc = ""]

//! Draw adic visualizations
//!
//! #### 7-adic sqrt(2)
#![doc = ""]
#![doc = include_str!("../img/clock-7-sqrt-2.svg")]
#![doc = include_str!("../img/tree-7-sqrt-2.svg")]
#![doc = include_str!("../img/euclidean-7-sqrt-2.svg")]
#![doc = ""]

//! # Adic visualizations
//!
//! - Clocks
//! - Trees
//! - Euclideans (2D fractals)
//!
//! ## Links
//! - [crates.io/adic-shape](<https://crates.io/crates/adic-shape>)
//! - [docs/adic-shape](https://docs.rs/adic-shape/latest/adic_shape/)
//! - [gitlab/adic](https://gitlab.com/saplingcalculations/adic)
//! - [adicmath.com](https://adicmath.com) - learn about and play with adic numbers
//! - [crates.io/adic](<https://crates.io/crates/adic>) - base crate for adic math (no visualization)
//! - [wiki/P-adic_number](<https://en.wikipedia.org/wiki/P-adic_number>) - wikipedia for p-adic numbers
//!
//!
//! ## Motivation
//!
//! p-adic numbers are an alternate number system to the reals, containing the rationals.
//! Digitally, these numbers are a number expansion where there can be an infinite numbers
//!  to the LEFT of the decimal point instead of the right.
//! Assume for this documentation that `p=5`, i.e. the digits of the adic number are
//!  `0`, `1`, `2`, `3`, or `4`.
//!
//! Examples:
//! - 0 = 0._5
//! - 1 = 1._5
//! - 2 = 2._5
//! - 5 = 10._5
//! - 25 = 100._5
//! - 53 = 203._5
//! - -1/4 = ...111._5
//! - 3/4 = ...112._5
//! - 7-adic 1st sqrt(2) = ...6623164112011266421216213._7
//! - 7-adic 2nd sqrt(2) = ...0043502554655400245450454._7
//!
//! ## Adic clock visualization
//!
//! Given this digital expansion, you can construct a visualization in the form of a clock.
//! Assume `p=5`.
//! Imagine an analog clock, but with two differences:
//! - There are `p=5` tick marks instead of `12` or `60`.
//! - There are an infinite number of hands.
//! - There are `p=5` seconds per minute, `p=5` minutes per hour, `p=5` hours per next increment, and so on.
//!
//! For each digit from the "ones" place onward, set the corresponding hand to the corresponding tick mark.
//! It makes more sense as a "sweeping clock", e.g. if the ones place is `3` and the fives place is `1`,
//!  then set the first hand to the `3` tick and the second hand to `3/5` past the `1` tick mark.

#![doc = ""]
#![doc = include_str!("../img/clock-158.svg")]
#![doc = ""]

//! ## Adic tree visualization
//!
//! Similarly, you can construct a visualization in the form of a tree.
//! Imagine a tree growing from a root with `p=5` branches.
//! Each of the branches has five branches, and each of those has five, etc.
//!
//! You can associate an adic number with an infinite path from the root of this tree to the top.
//! Each digit of the number is a choice at each branch point upward,
//!  from the "ones" place to the "fives" to the "twenty-fives" and so on.
//! E.g. the number `158 = ...00001113._5` has a choice of the "third" branch and then
//!  "one", "one", "one", and then zeros infinitely upward.

#![doc = ""]
#![doc = include_str!("../img/full-tree-158.svg")]
#![doc = ""]

//! You can also plot a "zoomed-in" version that focuses just on the chosen branches.
//! (This is usually better for performance.)

#![doc = ""]
#![doc = include_str!("../img/zoomed-tree-158.svg")]
#![doc = ""]

//! ## Adic euclidean visualization
//!
//! This visualization is more complicated.
//! It something like combining the clock and tree visualizations together.
//! We need one more parameter for this visualization: `scaling`.
//!
//! Roughly, think of this as the clock, but where each hand extends from the end of the previous hand.
//! We scale down the clock hands with `scaling` each time.
//! So the first hand is length `1`, the second length `1/scaling`, the third `1/scaling^2`, etc.
//! The clock is still a "sweeping" clock, so if the first hand points to `3`, then the second hand will point in the `d + 3/p` direction.
//!
//! In this way, it's clock-like because you are using the evenly distributed clock hand angles
//!  but tree-like because you are extending branches from the ends of the previous branches.
//!
//! If you visualize *all* numbers in this way, you will create a fractal.
//! We tend to call this the "characteristic p-adic euclidean" or "characteristic fractal".

#![doc = ""]
#![doc = include_str!("../img/full-euclidean-158.svg")]
#![doc = ""]

//! Similarly to the tree, it is easier on the computer to just focus on specific adic numbers rather than drawing an entire fractal in svg.
//!
//! Here are plots of all roots of unity in the 5-adic space `Z_5`,
//!  `{...000001._5 = 1, ...431212._5, ...013233._5, ...444444._5 = -1}`.
//! The first shows the full fractal with these roots colored and the second just shows the roots.

#![doc = ""]
#![doc = include_str!("../img/full-euclidean-roots-of-unity.svg")]
#![doc = include_str!("../img/euclidean-roots-of-unity.svg")]
#![doc = ""]

//! We also support more traditional fractal representations, e.g. the Sierpinsky triangle for the 3-adics.
//! But this characteristic fractal is a little more natural, handling addition of adic numbers more smoothly without discontinuities.
//!
//! ## Crate
//!
//! Currently this crate creates both svg documents and leptos components.
//! The leptos components are slightly better tested.
//!
//! The display-independent code is in the `shape` module and the `leptos` and `svg` modules have the code for those displays.
//! The [`DisplayShape`](shape::DisplayShape) trait provides the interface between display-independent and display code.
//! We are oriented around a svg display implementation, since both `leptos` and `svg` are html svg displays.
//!
//! To create a shape, set up an appropriate [`AdicCanvas`](shape::AdicCanvas),
//!  e.g. [`ClockCanvas`](shape::ClockCanvas), [`TreeCanvas`](shape::TreeCanvas), [`EuclideanCanvas`](shape::EuclideanCanvas).
//! Once you have chosen all of the display options in the canvas, draw a shape with the canvas methods:
//! - [`draw_integer`](shape::AdicCanvas::draw_integer) - Draw a single adic integer
//! - [`draw_integers`](shape::AdicCanvas::draw_integers) - Draw multiple adic integers
//! - [`draw_number`](shape::AdicCanvas::draw_number) - Draw a single adic number
//! - [`draw_numbers`](shape::AdicCanvas::draw_numbers) - Draw multiple adic numbers
//! - [`draw_full`](shape::AdicCanvas::draw_full) - Draw full p-adic integer space, `Z_p`
//!
//! These methods are not all supported by all canvases.
//! For example, clocks only draw single integers or numbers, because drawing multiple numbers on the same clock face is not too useful.
//!
//! After you draw the shape, you can display it as SVG or as leptos component.
//! To print or save a raw SVG, use the [`SvgDisplay`](svg::SvgDisplay) trait.
//! To create a component for use in your `leptos` application,
//!  install this crate with the `leptos` feature flag
//!  and feed the created shape into [`ShapeComponent`](leptos::ShapeComponent) or [`ShapeCard`](leptos::ShapeCard).
//!
//! Deploy the dev suite to try the examples to see how to use the leptos components.
//! First, install [`trunk`](https://trunk-rs.github.io/trunk/),
//!  running `cargo install --locked trunk` or following [this guide](https://trunk-rs.github.io/trunk/guide/getting-started/installation.html).
//! Then run `trunk serve` in the `adic-shape` directory.
//! This will deploy the [testing suite](https://gitlab.com/saplingcalculations/adic/-/tree/develop/adic-shape/dev/),
//!  accessible by default at <http://localhost:8080>.
//! Look through the test suites for components you would like to use.
//!
//! #### Save clock/tree/euclidean SVG
//!
//! ```no_run
//! # use adic::{traits::PrimedFrom, EAdic, ZAdic};
//! # use adic_shape::{shape::{AdicCanvas, ClockCanvas, EuclideanCanvas, Direction, TreeCanvas}, svg::SvgDisplay};
//! // Draw clock SVG for 5-adic -158
//! let depth = 25;
//! let neg_158 = EAdic::primed_from(5, -158);
//! let canvas = ClockCanvas::builder().base(5).depth(depth).build();
//! let clock_shape = canvas.draw_integer(&neg_158)?;
//! let svg_doc = clock_shape.create_svg_doc();
//! svg::save("image.svg", &svg_doc)?;
//!
//! // Draw zoomed tree SVG for 5-adic -1/4
//! let depth = 25;
//! let neg_one_fourth = EAdic::new_repeating(5, vec![], vec![1]);
//! let canvas = TreeCanvas::builder()
//!     .base(5)
//!     .depth(depth)
//!     .direction(Direction::Up)
//!     .dangling_direction(Some(Direction::Down))
//!     .build();
//! let tree_shape = canvas.draw_integer(&neg_one_fourth)?;
//! let svg_doc = tree_shape.create_svg_doc();
//! svg::save("image.svg", &svg_doc)?;
//!
//! // Draw euclidean SVG for 5-adic roots of unity
//! let depth = 5;
//! let scaling = 3.6;
//! let canvas = EuclideanCanvas::builder()
//!     .characteristic_p_adic(5)
//!     .scaling(scaling).depth(depth)
//!     .draw_scaled_hulls()
//!     .solid_full_tree()
//!     .build();
//! let roots_precision = 5; // Must be >= depth
//! let roots_of_unity_variety  = ZAdic::roots_of_unity(5, roots_precision)?;
//! let roots_of_unity = roots_of_unity_variety.roots();
//! let euclidean_shape = canvas.draw_integers(roots_of_unity)?;
//! let svg_doc = euclidean_shape.create_svg_doc();
//! svg::save("image.svg", &svg_doc)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! #### Draw clock/tree/euclidean leptos component
//!
//! ```no_run
//! # use adic::{traits::PrimedFrom, EAdic, QAdic, ZAdic};
//! # use adic_shape::{leptos::ShapeCard, shape::{AdicCanvas, ClockCanvas, Direction, EuclideanCanvas, TreeCanvas}};
//! # use leptos::prelude::*;
//! # use num::Rational32;
//! // Create clock view for 5-adic 4321._5 = 586
//! let depth = 6;
//! let five_eighty_six = EAdic::new(5, vec![1, 2, 3, 4]);
//! let clock_canvas = ClockCanvas::builder().base(5).depth(depth).build();
//! let clock_shape = clock_canvas.draw_integer(&five_eighty_six)?;
//! let clock_view = view! {
//!     <ShapeCard class="clock" shape=clock_shape/>
//! };
//!
//! // Create tree view for 7-adic 312612._7 ~= sqrt(2)
//! let depth = 6;
//! let approx_7_adic_sqrt_2 = ZAdic::new_approx(7, 6, vec![3, 1, 2, 6, 1, 2]);
//! let canvas = TreeCanvas::builder()
//!     .base(7)
//!     .depth(depth)
//!     .direction(Direction::Right)
//!     .dangling_direction(Some(Direction::Down))
//!     .build();
//! let tree_shape = canvas.draw_integer(&approx_7_adic_sqrt_2)?;
//! let tree_view = view! {
//!     <ShapeCard class="tree" shape=tree_shape/>
//! };
//!
//! // Create euclidean view for 11-adic 34/11
//! let depth = 6;
//! let scaling = 3.6;
//! let thirty_four_elevenths = QAdic::<EAdic>::primed_from(11, Rational32::new(34, 11));
//! let canvas = EuclideanCanvas::builder()
//!     .characteristic_p_adic(11)
//!     .scaling(scaling).depth(depth)
//!     .build();
//! let euclidean_shape = canvas.draw_number(&thirty_four_elevenths)?;
//! let euclidean_view = view! {
//!     <ShapeCard class="euclidean" shape=euclidean_shape/>
//! };
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## `adic`
//!
//! The [`adic`](https://crates.io/crates/adic) crate handles the math of adic numbers.
//! The primary intention of the `adic-shape` crate is for visualizations of these numbers.
//! Combining the two crates is how to get the best out of `adic-shape`.
//!
//! E.g. you can calculate the 7-adic +/- sqrt(2) with the `adic` crate:
//!
//! ```
//! # use adic::{traits::AdicInteger, EAdic, Variety, ZAdic};
//! let seven_adic_sqrt_2_variety = EAdic::new(7, vec![2]).nth_root(2, 6);
//! assert_eq!(
//!     Ok(Variety::new(vec![
//!         ZAdic::new_approx(7, 6, vec![3, 1, 2, 6, 1, 2]),
//!         ZAdic::new_approx(7, 6, vec![4, 5, 4, 0, 5, 4]),
//!     ])),
//!     seven_adic_sqrt_2_variety
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Then you can plot these calculated numbers as above.
//!
//! ## TODO
//!
//! - Interactive visualizations (drag the clock hands to change numbers)
//! - Visualizing higher adic spaces, e.g. finite extensions and complex adic numbers


#![cfg_attr(docsrs, feature(doc_cfg))]


// RE-EXPORTS

/// Re-export `adic` crate
pub use adic;
/// Re-export `num` crate
pub use num;


// MACROS
// None


// MODULES

// Public modules
pub mod error;
pub mod svg;

#[cfg(feature="leptos")]
#[cfg_attr(docsrs, doc(cfg(feature = "leptos")))]
pub mod leptos;

// Future public modules
// None

// Private modules
mod draw;

// Public submodules
pub use draw::animation;
pub use draw::interactive;
pub use draw::shape;


// EXPORTS (enum, struct, trait, type)

// Public exports
// None

// Future public exports
// None

// Private exports
// None
