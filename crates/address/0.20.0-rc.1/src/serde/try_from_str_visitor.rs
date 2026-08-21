use serde::de::{Error, Unexpected, Visitor};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// A serde visitor that parses a string with `TryFrom`.
pub(crate) struct TryFromStrVisitor<'de, T: TryFrom<&'de str>> {
    expecting: &'static str,
    phantom: PhantomData<fn(&'de str) -> T>,
}

impl<'de, T: TryFrom<&'de str>> TryFromStrVisitor<'de, T> {
    //! Construction

    /// Creates a new visitor with the `expecting` message.
    pub(crate) const fn new(expecting: &'static str) -> Self {
        Self {
            expecting,
            phantom: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for TryFromStrVisitor<'de, T>
where
    T: TryFrom<&'de str>,
    T::Error: Display,
{
    type Value = T;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.expecting)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        T::try_from(v).map_err(E::custom)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match std::str::from_utf8(v) {
            Ok(s) => self.visit_borrowed_str(s),
            Err(_) => Err(E::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }
}
