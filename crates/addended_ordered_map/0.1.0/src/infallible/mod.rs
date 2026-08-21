/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

//! The infallible (and more common) version of this mapping.
//!
//! Assumes size and key addition operations do not fail.
//!
//! Refer to the documentation of [`AddendedOrderedMap`] for more info.
//!
//! [`AddendedOrderedMap`]: crate::infallible::AddendedOrderedMap

mod addendable_key;
mod addended_ordered_map;
mod sized_value;

pub use self::addendable_key::AddendableKey;
pub use self::addended_ordered_map::AddendedOrderedMap;
pub use self::sized_value::SizedValue;
