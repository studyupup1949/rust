mod stateful_callback;
pub use stateful_callback::StatefulCallback;

mod async_stateful_callback;
pub use async_stateful_callback::AsyncStatefulCallback;

#[cfg(test)]
mod tests;
