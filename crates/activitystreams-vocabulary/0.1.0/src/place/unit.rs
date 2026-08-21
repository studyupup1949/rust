use serde::{Deserialize, Serialize};

use crate::{Error, Iri, Result, impl_default, impl_display};

/// Specifies the measurement units for the `radius` and `altitude` properties on a [Place](crate::Place) object.
///
/// If not specified, the default is assumed to be "m" for "meters".
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Unit {
    #[serde(rename = "cm")]
    Centimeters,
    #[serde(rename = "m")]
    Meters,
    #[serde(rename = "km")]
    Kilometers,
    #[serde(rename = "inches")]
    Inches,
    #[serde(rename = "feet")]
    Feet,
    #[serde(rename = "miles")]
    Miles,
}

impl Unit {
    pub const CENTIMETERS: &str = "cm";
    pub const METERS: &str = "m";
    pub const KILOMETERS: &str = "km";
    pub const INCHES: &str = "inches";
    pub const FEET: &str = "feet";
    pub const MILES: &str = "miles";

    /// Creates a new [Unit].
    #[inline]
    pub const fn new() -> Self {
        Self::Meters
    }

    /// Gets the string representation of the [Unit].
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Centimeters => Self::CENTIMETERS,
            Self::Meters => Self::METERS,
            Self::Kilometers => Self::KILOMETERS,
            Self::Inches => Self::INCHES,
            Self::Feet => Self::FEET,
            Self::Miles => Self::MILES,
        }
    }
}

impl_default!(Unit);
impl_display!(Unit, str);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Units {
    Unit(Unit),
    Iri(Iri),
}

impl Units {
    /// Creates a new [Units].
    pub const fn new() -> Self {
        Self::Unit(Unit::new())
    }

    /// Creates a new [Units] [Unit](Self::Unit) variant.
    pub fn unit<I: Into<Unit>>(val: I) -> Self {
        Self::Unit(val.into())
    }

    /// Gets whether the [Units] contains a [Unit](Self::Unit) variant.
    pub const fn is_unit(&self) -> bool {
        matches!(self, Self::Unit(_))
    }

    /// Attempts to get the [Unit](Self::Unit) variant.
    pub fn as_unit(&self) -> Result<Unit> {
        match self {
            Self::Unit(unit) => Ok(*unit),
            _ => Err(Error::place("invalid place type")),
        }
    }

    /// Creates a new [Units] [Iri](Self::Iri) variant.
    pub fn iri<I: Into<Iri>>(val: I) -> Self {
        Self::Iri(val.into())
    }

    /// Gets whether the [Units] contains a [Iri](Self::Iri) variant.
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to get a reference to the [Iri](Self::Iri) variant.
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(iri) => Ok(iri),
            _ => Err(Error::place("invalid place type")),
        }
    }
}

impl From<Unit> for Units {
    fn from(val: Unit) -> Self {
        Self::Unit(val)
    }
}

impl From<Iri> for Units {
    fn from(val: Iri) -> Self {
        Self::Iri(val)
    }
}
