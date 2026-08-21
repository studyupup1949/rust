mod once;
pub use once::Once;

mod async_once;
pub use async_once::AsyncOnce;

#[cfg(test)]
mod tests;
