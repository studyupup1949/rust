use num::{rational::Ratio, Num};
use crate::error::{AdicError, AdicResult};


/// A ring that can represent a finite valuation
pub trait ValuationRing: Clone + Copy + PartialOrd + Ord + Num {

    /// Convert `usize` to `ValuationRing`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. usize -> isize
    fn try_from_usize(val: usize) -> AdicResult<Self>;
    /// Convert `ValuationRing` to `usize`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. isize -> usize
    fn try_into_usize(self) -> AdicResult<usize>;

    /// Convert `isize` to `ValuationRing`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. isize -> usize
    fn try_from_isize(val: isize) -> AdicResult<Self>;
    /// Convert `ValuationRing` to `isize`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. usize -> isize
    fn try_into_isize(self) -> AdicResult<isize>;

    /// Convert `u32` to `ValuationRing`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. u32 -> isize
    fn try_from_u32(val: u32) -> AdicResult<Self> {
        Self::try_from_usize(usize::try_from(val)?)
    }
    /// Convert `ValuationRing` to `u32`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. isize -> u32
    fn try_into_u32(self) -> AdicResult<u32> {
        Ok(self.try_into_usize()?.try_into()?)
    }

    /// Convert `i32` to `ValuationRing`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. i32 -> usize
    fn try_from_i32(val: i32) -> AdicResult<Self> {
        Self::try_from_isize(isize::try_from(val)?)
    }
    /// Convert `ValuationRing` to `i32`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. usize -> i32
    fn try_into_i32(self) -> AdicResult<u32> {
        Ok(self.try_into_isize()?.try_into()?)
    }

}
impl ValuationRing for usize {
    fn try_from_usize(val: usize) -> AdicResult<Self> {
        Ok(val)
    }
    fn try_into_usize(self) -> AdicResult<usize> {
        Ok(self)
    }
    fn try_from_isize(val: isize) -> AdicResult<Self> {
        Ok(val.try_into()?)
    }
    fn try_into_isize(self) -> AdicResult<isize> {
        Ok(self.try_into()?)
    }
}
impl ValuationRing for isize {
    fn try_from_usize(val: usize) -> AdicResult<Self> {
        Ok(val.try_into()?)
    }
    fn try_into_usize(self) -> AdicResult<usize> {
        Ok(self.try_into()?)
    }
    fn try_from_isize(val: isize) -> AdicResult<Self> {
        Ok(val)
    }
    fn try_into_isize(self) -> AdicResult<isize> {
        Ok(self)
    }
}
impl ValuationRing for Ratio<usize> {
    fn try_from_usize(val: usize) -> AdicResult<Self> {
        Ok(Ratio::from_integer(val))
    }
    fn try_into_usize(self) -> AdicResult<usize> {
        if self.is_integer() {
            Ok(self.to_integer())
        } else {
            Err(AdicError::BadConversion)
        }
    }
    fn try_from_isize(val: isize) -> AdicResult<Self> {
        Ok(Ratio::from_integer(val.try_into_usize()?))
    }
    fn try_into_isize(self) -> AdicResult<isize> {
        if self.is_integer() {
            Ok(self.to_integer().try_into_isize()?)
        } else {
            Err(AdicError::BadConversion)
        }
    }
}
impl ValuationRing for Ratio<isize> {
    fn try_from_usize(val: usize) -> AdicResult<Self> {
        Ok(Ratio::from_integer(val.try_into()?))
    }
    fn try_into_usize(self) -> AdicResult<usize> {
        if self.is_integer() {
            Ok(self.to_integer().try_into()?)
        } else {
            Err(AdicError::BadConversion)
        }
    }
    fn try_from_isize(val: isize) -> AdicResult<Self> {
        Ok(Ratio::from_integer(val))
    }
    fn try_into_isize(self) -> AdicResult<isize> {
        if self.is_integer() {
            Ok(self.to_integer())
        } else {
            Err(AdicError::BadConversion)
        }
    }
}
