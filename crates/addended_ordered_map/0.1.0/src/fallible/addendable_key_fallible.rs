/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/// A key type whose offset addition may fail.
pub trait AddendableKeyFallible<SIZE, E>
where
    Self: Sized,
{
    /// Adds a given size value to a key, producing the key type.
    fn add_size(&self, size: &SIZE) -> Result<Self, E>;
}
