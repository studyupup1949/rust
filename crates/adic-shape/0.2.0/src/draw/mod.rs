//! Drawing elements, independent of the display output, e.g. leptos or svg

pub mod animation;
pub (crate) mod element;
pub mod interactive;
pub mod shape;
pub (crate) mod util;

#[cfg(test)]
mod test_util;
