#![allow(clippy::too_many_lines)]

mod animate;
mod clock;
mod comparison;
mod euclidean;
mod hensel;
mod interactive;
mod operation;
mod real_projection_chart;
mod thaw;
mod tree;

pub use animate::AnimateSuite;
pub use clock::ClockSuite;
pub use comparison::ComparisonSuite;
pub use euclidean::EuclideanSuite;
pub use hensel::HenselSuite;
pub use interactive::InteractiveSuite;
pub use operation::OperationSuite;
pub use real_projection_chart::RealProjectionChartSuite;
pub use thaw::ThawSuite;
pub use tree::TreeSuite;
