#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod tables;

pub use tables::*;

/// Generic type for the subclasses of device classes.
///
/// This is a new type arround `u16`, no validity checking is performed.
///
/// If you are using a well known device class, you may want to use one of the well-known subclass enums.
#[cfg_attr(feature = "fixed-repr", repr(transparent))]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SubclassId(u16);

impl SubclassId {
    /// Obtains the appropriate [`SubclassId`] from the given numeric id.
    pub const fn from_id(x: u16) -> Self {
        Self(x)
    }

    /// Converts the [`SubclassId`] to the appropriate numeric id
    pub const fn id(self) -> u16 {
        self.0
    }
}

/// Generic type for the products of registered vendors
///
/// This is a new type arround `u16`, no validity checking is performed.
#[cfg_attr(feature = "fixed-repr", repr(transparent))]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProductId(u16);

impl ProductId {
    /// Obtains the appropriate [`ProductId`] from the given numeric id.
    pub const fn from_id(x: u16) -> Self {
        Self(x)
    }

    /// Converts the [`ProductId`] to the appropriate numeric id
    pub const fn id(self) -> u16 {
        self.0
    }
}
