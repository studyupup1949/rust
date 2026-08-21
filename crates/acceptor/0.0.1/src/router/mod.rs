mod router;
pub use router::{Router, RouterEntry};

mod async_router;
pub use async_router::{AsyncRouter, AsyncRouterEntry};

#[cfg(test)]
mod tests;
