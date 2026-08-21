/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

//! The fallible version of the mapping.
//!
//! Allows for fallible size and key addition operations where addend or size
//! computation may fail.
//!
//! Refer to the documentation of [`AddendedOrderedMapFallible`] for more info.
//!
//! [`AddendedOrderedMapFallible`]: crate::fallible::AddendedOrderedMapFallible

mod addendable_key_fallible;
mod addended_ordered_map_fallible;
mod sized_value_fallible;

pub use self::addendable_key_fallible::AddendableKeyFallible;
pub use self::addended_ordered_map_fallible::AddendedOrderedMapFallible;
pub use self::sized_value_fallible::SizedValueFallible;
