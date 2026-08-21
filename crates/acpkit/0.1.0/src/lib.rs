#![doc = include_str!("../README.md")]

/// Returns a short product description.
#[must_use]
pub fn description() -> &'static str {
    "ACP adapters and transport tooling for exposing existing agent runtimes through the Agent Client Protocol."
}
