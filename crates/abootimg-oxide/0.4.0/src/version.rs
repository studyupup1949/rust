use core::fmt;

use binrw::{BinRead, BinWrite};

/// OS version and patch level
///
/// # Warning
///
/// Please note that the version information may be incorrect, for example, on OnePlus devices.
///
/// # Bitwise format
///
/// * 7 bits indicate first part of version
/// * 7 bits indicate second part of version
/// * 7 bits indicate third part of version
/// * 12 bits indicate patch year
/// * 4 bits indicate patch month
#[derive(BinRead, BinWrite, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[br(little)]
pub struct OsVersionPatch(pub u32);

impl OsVersionPatch {
    /// Creates a new `OsVersionPatch`.
    #[must_use]
    pub const fn new(version: OsVersion, patch: OsPatch) -> Self {
        Self((version.0 << 11) + patch.0 as u32)
    }
    /// Returns the version part.
    #[must_use]
    pub const fn version(self) -> OsVersion {
        OsVersion(self.0 >> 11)
    }
    /// Returns the patch part.
    #[must_use]
    pub const fn patch(self) -> OsPatch {
        OsPatch((self.0 & 0x7ff) as u16)
    }
}

impl fmt::Debug for OsVersionPatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OsVersionPatch({}, {})", self.version(), self.patch())
    }
}

/// Error returned by [`OsPatch::new`].
#[non_exhaustive]
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OsPatchError {
    /// `year` under 2000, which is not supported by the format.
    #[error("`year` under 2000, which is not supported by the format.")]
    YearTooSmall,
    /// `year` over 6095, which is not supported by the format (`year - 2000` won't fit into 12 bits).
    #[error("`year` over 6095, which is not supported by the format (`year - 2000` won't fit into 12 bits).")]
    YearWontFit,
    #[error("`month` over 15, which is not supported by the format (won't fit into 4 bits).")]
    MonthWontFit,
}

/// OS patch level
///
/// # Bitwise format
///
/// * 12 bits indicate year
/// * 4 bits indicate month
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OsPatch(pub u16);
impl OsPatch {
    /// Creates a new `OsPatch`.
    ///
    /// # Errors
    ///
    /// Returns `Err` when either of the following occur:
    ///
    /// - `year` is under 2000
    /// - `year` is over 6095 (`year - 2000` won't fit into 12 bits)
    /// - `month` is over 15 (`month` won't fit into 4 bits)
    pub const fn new(year: u16, month: u8) -> Result<Self, OsPatchError> {
        const U12_MAX: u16 = 2_u16.pow(12) - 1;
        const U4_MAX: u8 = 2_u8.pow(4) - 1;

        let Some(years_after_2000) = year.checked_sub(2000) else {
            return Err(OsPatchError::YearTooSmall);
        };

        if years_after_2000 > U12_MAX {
            return Err(OsPatchError::YearWontFit);
        }

        if month > U4_MAX {
            return Err(OsPatchError::MonthWontFit);
        }

        Ok(Self((years_after_2000 << 4) + month as u16))
    }
    /// Returns the year.
    #[must_use]
    pub const fn year(self) -> u16 {
        // Highest 12 bits indicate year
        (self.0 >> 4) + 2000
    }
    /// Returns the month.
    #[must_use]
    pub const fn month(self) -> u8 {
        // Lowest 4 bits indicate month
        (self.0 & 0xf) as u8
    }
}

impl fmt::Display for OsPatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:02}", self.year(), self.month())
    }
}
impl fmt::Debug for OsPatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Error returned by [`OsVersion::new`].
///
/// "A version component is over 127, which is not supported by the format (won't fit into 7 bits)."
#[derive(thiserror::Error, Debug, PartialEq)]
#[error("A version component is over 127, which is not supported by the format (won't fit into 7 bits).")]
pub struct OsVersionWontFitError;

/// OS version
///
/// # Warning
///
/// Please note that this information may be incorrect, for example, on OnePlus devices.
///
/// # Bitwise format
///
/// * 7 bits indicate first part
/// * 7 bits indicate second part
/// * 7 bits indicate third part
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OsVersion(pub u32);
impl OsVersion {
    /// Creates a new `OsVersion`.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if a version component is over 127, which means it won't fit into
    /// 7 bits.
    pub const fn new(a: u8, b: u8, c: u8) -> Result<Self, OsVersionWontFitError> {
        const U7_MAX: u8 = 2_u8.pow(7) - 1;

        if a > U7_MAX || b > U7_MAX || c > U7_MAX {
            return Err(OsVersionWontFitError);
        }

        Ok(Self(((a as u32) << 14) | ((b as u32) << 7) | c as u32))
    }
    /// Returns the version parts.
    #[must_use]
    pub const fn version_parts(self) -> (u8, u8, u8) {
        let x = self.0;
        let a = (x >> 14) & 0x7f; // Top 7 bits
        let b = (x >> 7) & 0x7f; // Middle 7 bits
        let c = x & 0x7f; // Low 7 bits
        (a as u8, b as u8, c as u8)
    }
}

impl fmt::Display for OsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (a, b, c) = self.version_parts();
        write!(f, "{a}.{b}.{c}")
    }
}
impl fmt::Debug for OsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{format, string::ToString as _};

    use super::*;

    #[test]
    fn basic() {
        let vp = OsVersionPatch(0x1800_0186);
        assert_eq!(format!("{vp:?}"), "OsVersionPatch(12.0.0, 2024-06)");
        assert_eq!(vp.version().to_string(), "12.0.0");
        assert_eq!(vp.patch().to_string(), "2024-06");
        assert_eq!(vp, OsVersionPatch::new(vp.version(), vp.patch()));
        assert_eq!(Ok(vp.version()), OsVersion::new(12, 0, 0));
        assert_eq!(Ok(vp.patch()), OsPatch::new(2024, 6));
    }

    #[test]
    fn truncating_behavior() {
        let ver = OsVersion(0b1111_1111 << 14);
        assert_eq!(ver.version_parts(), (0b0111_1111, 0, 0));
        assert_eq!(format!("{ver:?}"), "127.0.0");
        assert_eq!(ver.to_string(), "127.0.0");
    }

    #[test]
    fn errors() {
        assert_eq!(OsVersion::new(0, 0, 0), Ok(OsVersion(0)));
        assert_eq!(OsVersion::new(128, 0, 0), Err(OsVersionWontFitError));
        assert_eq!(OsVersion::new(0, 128, 0), Err(OsVersionWontFitError));
        assert_eq!(OsVersion::new(0, 0, 128), Err(OsVersionWontFitError));

        assert_eq!(OsPatch::new(0, 0), Err(OsPatchError::YearTooSmall));
        assert_eq!(OsPatch::new(1999, 0), Err(OsPatchError::YearTooSmall));
        OsPatch::new(2000, 0).unwrap();

        OsPatch::new(6095, 0).unwrap();
        assert_eq!(OsPatch::new(6096, 0), Err(OsPatchError::YearWontFit));

        OsPatch::new(2000, 15).unwrap();
        assert_eq!(OsPatch::new(2000, 16), Err(OsPatchError::MonthWontFit));
    }
}
