#![doc = include_str!("../README.md")]

/// Returns the current status of this crate.
#[must_use]
pub fn status() -> &'static str {
    "acpremote is currently implemented as a Python package. See https://vcoderun.github.io/acpkit/acpremote/."
}
