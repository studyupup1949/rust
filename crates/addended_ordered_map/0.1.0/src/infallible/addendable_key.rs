/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::ops::Add;

use crate::fallible::AddendableKeyFallible;

/// A key that can be added with a size to know how big a range is.
pub trait AddendableKey<SIZE>
where
    Self: Sized,
{
    /// Adds a given size value to a key, producing the key type.
    fn add_size(&self, size: &SIZE) -> Self;
}

impl<K, S, E> AddendableKeyFallible<S, E> for K
where
    K: AddendableKey<S>,
{
    fn add_size(&self, size: &S) -> Result<Self, E> {
        Ok(AddendableKey::add_size(self, size))
    }
}

impl<K, S> AddendableKey<S> for K
where
    K: Copy + Add<S, Output = K>,
    S: Copy,
{
    fn add_size(&self, size: &S) -> Self {
        *self + *size
    }
}
