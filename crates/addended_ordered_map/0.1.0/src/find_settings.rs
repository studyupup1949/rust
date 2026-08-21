/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/// Allows configuring how lookups and insertions are performed in a mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FindSettings {
    pub(crate) allow_addend: bool,
}

impl FindSettings {
    /// Constructs a `FindSettings`.
    ///
    /// If `allow_addend` is `true` then mapping operations are ranged.
    /// Otherwise operations are done on the exact key instead.
    #[must_use]
    pub const fn new(allow_addend: bool) -> Self {
        Self { allow_addend }
    }

    /// Gets the `allow_addend` value used to construct this.
    #[must_use]
    pub const fn allow_addend(&self) -> bool {
        self.allow_addend
    }
}

impl Default for FindSettings {
    fn default() -> Self {
        Self::new(true)
    }
}
