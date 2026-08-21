pub mod primitives;
pub mod responses;
pub mod adoptium;
pub mod endpoints;

pub mod prelude {
    pub use super::primitives::prelude::*;
    pub use super::adoptium::Adoptium;
    pub use super::endpoints::*;
}
