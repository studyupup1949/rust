pub(self) mod internal;

mod merge_captured_params;
mod merge_generic_params;
mod merge_generics;
mod merge_lifetimes;
mod merge_type_param_bounds;
mod merge_where_predicates;

pub use merge_captured_params::merge_captured_params;
pub use merge_generic_params::merge_generic_params;
pub use merge_generics::merge_generics;
pub use merge_lifetimes::merge_lifetimes;
pub use merge_type_param_bounds::merge_type_param_bounds;
pub use merge_where_predicates::merge_where_predicates;
