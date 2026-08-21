/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::fallible::SizedValueFallible;

/// A value type that can report its size.
pub trait SizedValue<SIZE> {
    /// Gets the size associated the value of a pairing.
    ///
    /// The size type may be different to the type of the value itself.
    fn size(&self) -> SIZE;
}

impl<T, S, E> SizedValueFallible<S, E> for T
where
    T: SizedValue<S>,
{
    fn size(&self) -> Result<S, E> {
        Ok(SizedValue::size(self))
    }
}

impl<S> SizedValue<S> for S
where
    S: Copy,
{
    fn size(&self) -> S {
        *self
    }
}

impl<S> SizedValue<S> for Option<S>
where
    S: Copy + Default,
{
    fn size(&self) -> S {
        self.unwrap_or_default()
    }
}

impl<S> SizedValue<S> for (S,)
where
    S: SizedValue<S>,
{
    fn size(&self) -> S {
        self.0.size()
    }
}

impl<S, T1> SizedValue<S> for (S, T1)
where
    S: SizedValue<S>,
{
    fn size(&self) -> S {
        self.0.size()
    }
}

impl<S, T1, T2> SizedValue<S> for (S, T1, T2)
where
    S: SizedValue<S>,
{
    fn size(&self) -> S {
        self.0.size()
    }
}

impl<S, T1, T2, T3> SizedValue<S> for (S, T1, T2, T3)
where
    S: SizedValue<S>,
{
    fn size(&self) -> S {
        self.0.size()
    }
}

impl<S, T1, T2, T3, T4> SizedValue<S> for (S, T1, T2, T3, T4)
where
    S: SizedValue<S>,
{
    fn size(&self) -> S {
        self.0.size()
    }
}

impl<S, T1, T2, T3, T4, T5> SizedValue<S> for (S, T1, T2, T3, T4, T5)
where
    S: SizedValue<S>,
{
    fn size(&self) -> S {
        self.0.size()
    }
}

impl<S, T1, T2, T3, T4, T5, T6> SizedValue<S> for (S, T1, T2, T3, T4, T5, T6)
where
    S: SizedValue<S>,
{
    fn size(&self) -> S {
        self.0.size()
    }
}

impl<S, T1, T2, T3, T4, T5, T6, T7> SizedValue<S> for (S, T1, T2, T3, T4, T5, T6, T7)
where
    S: SizedValue<S>,
{
    fn size(&self) -> S {
        self.0.size()
    }
}

impl<S, T1, T2, T3, T4, T5, T6, T7, T8> SizedValue<S> for (S, T1, T2, T3, T4, T5, T6, T7, T8)
where
    S: SizedValue<S>,
{
    fn size(&self) -> S {
        self.0.size()
    }
}
