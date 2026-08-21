//! This crate provides extension traits that can be used for checking alignment of values.
//!
//! The trait [Align](crate::Align) is generic, and provides methods, that accept an alignment argument.
//! For the common alignment to a 4k boundary, the specialized trait [Align4k](crate::Align4k) is provided.
//!
//! # Examples
//! ```
//! use addr_align::Align;
//! use addr_align::Align4k;
//! assert_eq!(0xaabbaa.align_down(4096), 0xaab000);
//! assert_eq!(0xaabbaa.align4k_down(), 0xaab000);
//! ```
use std::fmt::Debug;
use std::ops::{
    Add,
    BitAnd,
    Not,
    Sub,
};

const ALIGNMENT4K: u16 = 1 << 12;

pub trait Align<A> {

    /// # Arguments
    ///
    /// * `alignment`: The alignment `self` will be aligned to.
    ///
    /// returns: `x` with alignment `alignment` such that `x >= self`
    ///
    /// # Examples
    ///
    /// ```
    /// # use addr_align::Align;
    /// let aligned = 0b110101.align_up(0b100);
    /// assert_eq!(aligned, 0b111000);
    /// ```
    fn align_up(
        self,
        alignment: A,
    ) -> Self;

    /// # Arguments
    ///
    /// * `alignment`: The alignment `self` will be aligned to.
    ///
    /// returns: `x` with alignment `alignment` such that `x <= self`
    ///
    /// # Examples
    ///
    /// ```
    /// # use addr_align::Align;
    /// let aligned = 0b110101.align_down(0b100);
    /// assert_eq!(aligned, 0b110100);
    /// ```
    fn align_down(
        self,
        alignment: A,
    ) -> Self;

    /// # Arguments
    ///
    /// * `alignment`: The alignment to check `self` against.
    ///
    /// returns: true, if `self` is aligned, and false otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use addr_align::Align;
    /// assert!(0b11100.is_aligned(0b100));
    /// assert!(!0b11110.is_aligned(0b100));
    /// ```
    fn is_aligned(
        &self,
        alignment: A,
    ) -> bool;
}

pub trait Align4k {
    ///
    /// returns: `x` aligned to 2^12 such that `x >= self`
    ///
    /// # Examples
    ///
    /// ```
    /// # use addr_align::Align4k;
    /// let aligned = 0xaaaaaa.align4k_up();
    /// assert_eq!(aligned, 0xaab000);
    /// ```
    fn align4k_up(self) -> Self;

    ///
    /// returns: `x` aligned to 2^12 such that `x <= self`
    ///
    /// # Examples
    ///
    /// ```
    /// # use addr_align::Align4k;
    /// let aligned = 0xaaaaaa.align4k_down();
    /// assert_eq!(aligned, 0xaaa000);
    /// ```
    fn align4k_down(self) -> Self;

    ///
    /// returns: true, if `self` is aligned to 2^12 and false otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use addr_align::Align4k;
    /// assert!(!0xaaaaaa.is_4kaligned());
    /// assert!(0xaaa000.is_4kaligned());
    /// ```
    fn is_4kaligned(&self) -> bool;
}

fn assert_valid_alignment<A>(alignment: A) where A: Sub<Output=A> + From<u8> + BitAnd<Output=A> + PartialEq + Copy {
    if alignment == A::from(0u8) {
        panic!("Alignment must not be zero");
    }
    if (alignment & (alignment - A::from(1u8))) != A::from(0u8) {
        panic!("Alignment must be a power of two");
    }
}

impl<T, A> Align<A> for T
    where
        T: Add<T, Output=T> + Not<Output=T> + From<A> + BitAnd<Output=T> + PartialEq + Copy,
        A: Sub<Output=A> + From<u8> + BitAnd<Output=A> + PartialEq + Copy,
{
    fn align_up(
        self,
        alignment: A,
    ) -> Self {
        assert_valid_alignment(alignment);
        let mask = T::from(alignment - A::from(1u8));
        (self + mask) & !mask
    }

    fn align_down(
        self,
        alignment: A,
    ) -> Self {
        assert_valid_alignment(alignment);
        self & !T::from(alignment - A::from(1u8))
    }

    fn is_aligned(
        &self,
        alignment: A,
    ) -> bool {
        assert_valid_alignment(alignment);
        let mask = T::from(alignment - A::from(1u8));
        (*self & mask) == T::from(A::from(0u8))
    }
}

impl<T> Align4k for T
    where
        T: Align<u16>,
{
    fn align4k_up(self) -> Self {
        self.align_up(ALIGNMENT4K)
    }

    fn align4k_down(self) -> Self {
        self.align_down(ALIGNMENT4K)
    }

    fn is_4kaligned(&self) -> bool {
        self.is_aligned(ALIGNMENT4K)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(0b111.align_up(1), 0b111);
        assert_eq!(0b111.align_up(2), 0b1000);
        assert_eq!(0b101.align_up(2), 0b110);
        assert_eq!(0xababab.align4k_up(), 0xabb000);
    }

    #[test]
    fn test_align_down() {
        assert_eq!(0b111.align_down(1), 0b111);
        assert_eq!(0b111.align_down(2), 0b110);
        assert_eq!(0b101.align_down(2), 0b100);
        assert_eq!(0xababab.align4k_down(), 0xaba000);
    }

    #[test]
    fn test_is_aligned() {
        assert!(0b111.is_aligned(1));
        assert!(!0b111.is_aligned(2));
        assert!(0b110.is_aligned(2));
        assert!(0xaba000.is_4kaligned());
        assert!(!0xababab.is_4kaligned());
    }

    #[test]
    #[should_panic]
    fn invalid_alignment_panics() {
        0b111.align_up(3);
    }
}