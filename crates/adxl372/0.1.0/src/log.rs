//! Logging helpers.

/// Shared log tag for defmt messages.
#[cfg(feature = "defmt")]
pub(crate) const LOG_TAG: &str = "[Adxl372]";
