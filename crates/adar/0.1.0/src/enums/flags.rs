//! [`Flags`] is a type-safe and verbose bitwise flag container.

use crate::prelude::{EnumVariant, ReflectEnum};
use num_traits::{One, PrimInt, Zero};
use std::ops::{BitAnd, BitOr, BitXor, Not, Sub};

/// Type-safe and verbose bitwise flag container.
/// The associated enum must be annotated with [`crate::macros::FlagEnum`] derive macro.
#[derive(Copy, Clone)]
pub struct Flags<E>(E::Type)
where
    E: ReflectEnum;

impl<E> Flags<E>
where
    E: ReflectEnum + Into<E::Type>,
    E::Type: FlagTypeConstraints,
{
    /// Creates a new [`Flags`] with no flags set.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C}
    ///
    /// assert_eq!(Flags::<MyFlags>::empty(), ());
    /// ```
    ///
    /// # Returns
    /// [`Flags`] with no flags set.
    #[inline(always)]
    pub fn empty() -> Self {
        Self(E::Type::zero())
    }

    /// Creates a new [`Flags`] with one flag set.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// #[derive(Debug)]
    /// enum MyFlags {A, B, C}
    ///
    /// assert_eq!(Flags::single(MyFlags::B), MyFlags::B);
    /// ```
    ///
    /// # Returns
    /// [`Flags`] with the specified flag set.
    #[inline(always)]
    pub fn single(value: E) -> Self {
        Self(value.into())
    }

    /// Creates a new [`Flags`] with all flags set.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C}
    ///
    /// let mut flag = Flags::<MyFlags>::full();
    /// assert_eq!(Flags::<MyFlags>::full(), MyFlags::A | MyFlags::B | MyFlags::C);
    /// ```
    ///
    /// # Returns
    /// [`Flags`] with all flags set.
    #[inline(always)]
    pub fn full() -> Self {
        Self(
            ((1 << E::count()) - 1)
                .try_into()
                .unwrap_or(E::Type::zero()),
        )
    }

    /// Sets the specified flags.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// #[derive(Debug)]
    /// enum MyFlags {A, B, C, D, E}
    ///
    /// let mut flag = Flags::empty();
    /// flag.set(MyFlags::A);
    /// assert_eq!(flag, MyFlags::A);
    /// flag.set(MyFlags::B | MyFlags::C);
    /// assert_eq!(flag, MyFlags::A | MyFlags::B | MyFlags::C);
    /// flag.set(Flags::full()); // Set all flags
    /// assert_eq!(flag, MyFlags::A | MyFlags::B | MyFlags::C | MyFlags::D | MyFlags::E);
    /// ```
    #[inline(always)]
    pub fn set(&mut self, flags: impl Into<Flags<E>>) {
        self.0 = self.0 | flags.into().0;
    }

    /// Resets the specified flags.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C, D, E}
    ///
    /// let mut flag = Flags::full();
    /// flag.reset(MyFlags::A);
    /// assert_eq!(flag, MyFlags::B | MyFlags::C | MyFlags::D | MyFlags::E);
    /// flag.reset(MyFlags::B | MyFlags::C);
    /// assert_eq!(flag, MyFlags::D | MyFlags::E);
    /// flag.reset(Flags::full()); // Reset all flags
    /// assert_eq!(flag, ());
    /// ```
    #[inline(always)]
    pub fn reset(&mut self, flags: impl Into<Flags<E>>) {
        self.0 = self.0 & !flags.into().0;
    }

    /// Toggles the specified flags.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C, D}
    ///
    /// let mut flag = MyFlags::A | MyFlags::B | MyFlags::C;
    /// flag.toggle(MyFlags::B);
    /// assert_eq!(flag, MyFlags::A | MyFlags::C);
    /// flag.toggle(MyFlags::B | MyFlags::C);
    /// assert_eq!(flag, MyFlags::A | MyFlags::B);
    /// ```
    #[inline(always)]
    pub fn toggle(&mut self, flags: impl Into<Flags<E>>) {
        self.0 = self.0 ^ flags.into().0;
    }

    /// Checks if all of the flags are set.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C}
    ///
    /// let flag = MyFlags::A | MyFlags::B;
    /// assert!(flag.all(MyFlags::A));
    /// assert!(!flag.all(MyFlags::B | MyFlags::C));
    /// assert!(!flag.all(MyFlags::C));
    /// ```
    ///
    /// # Returns
    /// `true` if all of the specified flags are set in `self`.
    #[inline(always)]
    pub fn all(&self, flags: impl Into<Flags<E>>) -> bool {
        let flags: Flags<E> = flags.into();
        self.0 & flags.0 == flags.0
    }

    /// Checks if any of the flags are set.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C}
    ///
    /// let mut flag = MyFlags::A | MyFlags::B;
    /// assert!(flag.any(MyFlags::A));
    /// assert!(flag.any(MyFlags::B | MyFlags::C));
    /// assert!(!flag.any(MyFlags::C));
    /// ```
    ///
    /// # Returns
    /// `true` if any of the specified flags are set in `self`.
    #[inline(always)]
    pub fn any(&self, flags: impl Into<Flags<E>>) -> bool {
        self.0 & flags.into().0 != E::Type::zero()
    }

    /// Creates a new [`Flags`] where both the flags from `self` and the specified flags are set.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C, D}
    ///
    /// let flags = (MyFlags::A | MyFlags::B).union(MyFlags::B | MyFlags::D);
    /// assert_eq!(flags, MyFlags::A | MyFlags::B | MyFlags::D)
    /// ```
    ///
    /// # Returns
    /// [`Flags`] with the union of the flags set.
    #[inline(always)]
    pub fn union(&self, flags: impl Into<Flags<E>>) -> Flags<E> {
        Self(self.0 | flags.into().0)
    }

    /// Creates a new [`Flags`] where only the flags both present in `self` and the specified flags are set.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// #[derive(Debug)]
    /// enum MyFlags {A, B, C, D}
    ///
    /// let flags = (MyFlags::A | MyFlags::B).intersect(MyFlags::B | MyFlags::D);
    /// assert_eq!(flags, MyFlags::B)
    /// ```
    ///
    /// # Returns
    /// [`Flags`] with the intersection of the flags set.
    #[inline(always)]
    pub fn intersect(&self, flags: impl Into<Flags<E>>) -> Flags<E> {
        Self(self.0 & flags.into().0)
    }

    /// Counts the number of flags set in `self`.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C, D}
    ///
    /// assert_eq!((MyFlags::A | MyFlags::B).len(), 2);
    /// assert_eq!(Flags::<MyFlags>::empty().len(), 0);
    /// assert_eq!(Flags::<MyFlags>::full().len(), 4);
    /// ```
    ///
    /// # Returns
    /// Number of flags set.
    pub fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Creates an iterator to iterate through the set flags.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// #[derive(Debug, Eq, PartialEq)]
    /// enum MyFlags {A, B, C, D}
    ///
    /// let flags = (MyFlags::A | MyFlags::B);
    /// let mut iter = flags.iter();
    /// assert_eq!(iter.next(), Some(&EnumVariant::new("A", Some(MyFlags::A))));
    /// assert_eq!(iter.next(), Some(&EnumVariant::new("B", Some(MyFlags::B))));
    /// ```
    ///
    /// # Returns
    /// An iterator.
    pub fn iter<'a>(&'a self) -> FlagsIterator<'a, E> {
        FlagsIterator::<E> {
            iter: E::variants().iter() as std::slice::Iter<'static, EnumVariant<E>>,
            flags: self,
        }
    }

    /// Checks if no flags are set.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C, D}
    ///
    /// assert!(Flags::<MyFlags>::empty().is_empty());
    /// assert!(!(MyFlags::A | MyFlags::B).is_empty());
    /// ```
    ///
    /// # Returns
    /// `true` if no flags are set.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0 == E::Type::zero()
    }

    /// Tries to create [`Flags`] from a raw value.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C, D}
    ///
    /// assert_eq!(Flags::<MyFlags>::try_from_raw(0b1010), Some(MyFlags::B | MyFlags::D));
    /// ```
    ///
    /// # Returns
    /// `Some` - [`Flags`] if the operation succeeds \
    /// `None` - Raw value contains out-of-range bits
    pub fn try_from_raw(raw: E::Type) -> Option<Self> {
        if raw & Self::full().0 != raw {
            None
        } else {
            Some(Self(raw))
        }
    }

    /// Converts `self` into a raw value.
    ///
    /// # Example
    /// ```
    /// use adar::prelude::*;
    ///
    /// #[FlagEnum]
    /// enum MyFlags {A, B, C, D}
    ///
    /// assert_eq!((MyFlags::B | MyFlags::D).into_raw(), 0b1010);
    /// ```
    ///
    /// # Returns
    /// Raw representation of [`Flags`]
    #[inline(always)]
    pub fn into_raw(self) -> E::Type {
        self.0
    }
}

impl<E, T> PartialEq<T> for Flags<E>
where
    E: ReflectEnum,
    E::Type: FlagTypeConstraints,
    T: Into<Self> + Copy,
{
    #[inline(always)]
    fn eq(&self, other: &T) -> bool {
        self.0 == (*other).into().0
    }
}

impl<E> std::fmt::Debug for Flags<E>
where
    E: ReflectEnum + Into<E::Type> + Copy + 'static,
    E::Type: FlagTypeConstraints,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        write!(f, "(")?;
        for flag in self.iter() {
            if !first {
                write!(f, ",")?;
            }
            write!(f, "{}", flag.name)?;
            first = false;
        }
        write!(f, ")")?;
        Ok(())
    }
}

/// Iterates set flags in a [`Flags`] container.
pub struct FlagsIterator<'a, E>
where
    E: ReflectEnum + 'static,
{
    iter: std::slice::Iter<'static, EnumVariant<E>>,
    flags: &'a Flags<E>,
}

impl<'a, E> Iterator for FlagsIterator<'a, E>
where
    E: ReflectEnum + Into<E::Type> + Copy,
    E::Type: FlagTypeConstraints,
{
    type Item = &'a EnumVariant<E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .by_ref()
            .find(|&flag| self.flags.any(flag.value.unwrap()))
    }
}

impl<E> BitOr<E> for Flags<E>
where
    E: ReflectEnum + Into<E::Type>,
    E::Type: FlagTypeConstraints,
{
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: E) -> Self::Output {
        let mut res = self;
        res.set(rhs);
        res
    }
}

impl<E> Default for Flags<E>
where
    E: ReflectEnum,
    E::Type: FlagTypeConstraints,
{
    #[inline(always)]
    fn default() -> Self {
        Self(E::Type::zero())
    }
}

impl<E> FromIterator<E> for Flags<E>
where
    E: ReflectEnum + Into<E::Type>,
    E::Type: FlagTypeConstraints,
{
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let mut result = Flags::<E>::empty();
        for flag in iter {
            result.set(flag);
        }
        result
    }
}

impl<E> From<E> for Flags<E>
where
    E: ReflectEnum + Into<E::Type>,
    E::Type: FlagTypeConstraints,
{
    #[inline(always)]
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl<E> From<()> for Flags<E>
where
    E: ReflectEnum + Into<E::Type>,
    E::Type: FlagTypeConstraints,
{
    #[inline(always)]
    fn from(_: ()) -> Self {
        Self::empty()
    }
}

#[cfg(feature = "serde")]
impl<E> serde::Serialize for Flags<E>
where
    E: ReflectEnum,
    E::Type: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'d, E> serde::Deserialize<'d> for Flags<E>
where
    E: ReflectEnum + Into<E::Type>,
    E::Type: FlagTypeConstraints + serde::Deserialize<'d>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        Flags::<E>::try_from_raw(E::Type::deserialize(deserializer)?)
            .ok_or(serde::de::Error::custom("Failed to deserialize flags"))
    }
}

#[doc(hidden)]
pub trait FlagTypeConstraints:
    Copy
    + Zero
    + One
    + PartialEq
    + Not<Output = Self>
    + Sub<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + TryFrom<usize>
    + PrimInt
{
}

impl<T> FlagTypeConstraints for T where
    T: Copy
        + Zero
        + One
        + PartialEq
        + Not<Output = T>
        + Sub<Output = T>
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + BitXor<Output = T>
        + TryFrom<usize>
        + PrimInt
{
}

#[cfg(test)]
mod test {
    use crate as adar;
    use crate::prelude::*;

    #[derive(Debug)]
    #[FlagEnum]
    enum TestEmpty {}

    #[derive(Debug)]
    #[FlagEnum]
    enum TestSmallU8 {
        F1,
        F2,
    }

    #[derive(Debug, Eq, PartialEq)]
    #[FlagEnum]
    enum TestU8 {
        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
    }
    #[derive(Debug, Eq, PartialEq)]
    #[FlagEnum]
    enum TestU16 {
        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
    }

    #[FlagEnum]
    #[repr(u64)]
    enum TestFlagsForced {
        F,
    }

    #[test]
    fn test_flag_default() {
        let flags = Flags::<TestU8>::default();
        assert!(!flags.any(TestU8::F1));
        assert!(!flags.any(TestU8::F2));
        assert!(!flags.any(TestU8::F3));
        assert!(flags.is_empty());
    }

    #[test]
    fn test_flag_empty() {
        let flags = Flags::<TestU8>::empty();
        assert!(!flags.any(TestU8::F1));
        assert!(!flags.any(TestU8::F2));
        assert!(!flags.any(TestU8::F3));
        assert!(flags.is_empty());
    }

    #[test]
    fn test_flag_full() {
        let flags = Flags::<TestU8>::full();
        assert!(flags.any(TestU8::F1));
        assert!(flags.any(TestU8::F2));
        assert!(flags.any(TestU8::F3));
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_flag_from_enum() {
        let flags = Flags::from(TestU8::F1);
        assert!(flags.any(TestU8::F1));
        assert!(!flags.any(TestU8::F2));
        let flags = Flags::from(TestU8::F2);
        assert!(!flags.any(TestU8::F1));
        assert!(flags.any(TestU8::F2));
    }

    #[test]
    fn test_flag_raw() {
        let flags = Flags::<TestU8>::try_from_raw(0b10).unwrap();
        assert_eq!(flags.into_raw(), 0b10);
        assert!(!flags.any(TestU8::F1));
        assert!(flags.any(TestU8::F2));
        assert!(!flags.any(TestU8::F3));
        let flags = Flags::<TestU8>::try_from_raw(0b111).unwrap();
        assert_eq!(flags.into_raw(), 0b111);
        assert!(flags.any(TestU8::F1));
        assert!(flags.any(TestU8::F2));
        assert!(flags.any(TestU8::F3));

        assert!(Flags::<TestSmallU8>::try_from_raw(0b11).is_some());
        assert!(Flags::<TestSmallU8>::try_from_raw(0b111).is_none());
        assert!(Flags::<TestU8>::try_from_raw(0b11111111).is_some());
    }

    #[test]
    fn test_flag_from_iter() {
        let flags = Flags::<TestU8>::from_iter([]);
        assert!(!flags.any(TestU8::F1));
        assert!(!flags.any(TestU8::F3));

        let flags = Flags::from_iter([TestU8::F1]);
        assert!(flags.any(TestU8::F1));
        assert!(!flags.any(TestU8::F3));

        let flags = Flags::from_iter([TestU8::F3]);
        assert!(!flags.any(TestU8::F1));
        assert!(flags.any(TestU8::F3));

        let flags = Flags::from_iter([TestU8::F1, TestU8::F3]);
        assert!(flags.any(TestU8::F1));
        assert!(flags.any(TestU8::F3));

        let flags = Flags::from_iter(vec![TestU8::F3]);
        assert!(!flags.any(TestU8::F1));
        assert!(flags.any(TestU8::F3));
    }

    #[test]
    fn test_flag_set() {
        let mut flags = Flags::<TestU8>::empty();
        flags.set(TestU8::F1);
        assert_eq!(flags, TestU8::F1);
        assert_ne!(flags, TestU8::F1 | TestU8::F2);
        flags.set(TestU8::F2);
        assert_eq!(flags, TestU8::F1 | TestU8::F2);
    }

    #[test]
    fn test_flag_reset() {
        let mut flags = TestU8::F1 | TestU8::F2 | TestU8::F7;
        assert_eq!(flags, TestU8::F1 | TestU8::F2 | TestU8::F7);
        flags.reset(TestU8::F2);
        assert_eq!(flags, TestU8::F1 | TestU8::F7);
        flags.reset(TestU8::F7);
        assert_eq!(flags, TestU8::F1);
        assert!(!flags.is_empty());
        flags.reset(TestU8::F1);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_flag_toggle() {
        let mut flags = Flags::<TestU8>::empty();
        flags.toggle(TestU8::F6);
        assert_eq!(flags, TestU8::F6);
        flags.toggle(TestU8::F2);
        assert_eq!(flags, TestU8::F2 | TestU8::F6);
        flags.toggle(TestU8::F2);
        assert_eq!(flags, TestU8::F6);
        flags.toggle(TestU8::F6);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_flag_all() {
        let flags = TestU16::F1 | TestU16::F3 | TestU16::F5 | TestU16::F7 | TestU16::F9;

        assert!(flags.all(Flags::empty()));
        assert!(flags.all(TestU16::F1));
        assert!(flags.all(TestU16::F1 | TestU16::F5));
        assert!(flags.all(TestU16::F1 | TestU16::F5 | TestU16::F9));
        assert!(!flags.all(TestU16::F4 | TestU16::F5));
        assert!(!flags.all(TestU16::F2 | TestU16::F5));
        assert!(!flags.all(TestU16::F2));
    }

    #[test]
    fn test_flag_any() {
        let flags = TestU16::F1 | TestU16::F3 | TestU16::F5 | TestU16::F7 | TestU16::F9;

        assert!(!flags.any(Flags::empty()));
        assert!(flags.any(TestU16::F1));
        assert!(flags.any(TestU16::F1 | TestU16::F5));
        assert!(flags.any(TestU16::F1 | TestU16::F5 | TestU16::F9));
        assert!(!flags.any(TestU16::F2 | TestU16::F4));
        assert!(!flags.any(TestU16::F2 | TestU16::F8));
        assert!(!flags.any(TestU16::F2));
    }

    #[test]
    fn test_flag_union() {
        let flags = (TestU16::F1 | TestU16::F3).union(TestU16::F5 | TestU16::F7 | TestU16::F9);
        assert_eq!(
            flags,
            TestU16::F1 | TestU16::F3 | TestU16::F5 | TestU16::F7 | TestU16::F9
        );
        assert_eq!(flags.union(flags), flags);
    }

    #[test]
    fn test_flag_intersect() {
        let flags = (TestU16::F1 | TestU16::F3).intersect(TestU16::F3 | TestU16::F7 | TestU16::F9);
        assert_eq!(flags, TestU16::F3);
        assert_eq!(flags.intersect(flags), flags);
    }

    #[test]
    fn test_flag_debug() {
        let flags = TestU16::F1 | TestU16::F3 | TestU16::F5 | TestU16::F7;
        assert_eq!(format!("{:?}", flags), "(F1,F3,F5,F7)".to_string());
    }

    #[test]
    fn test_flag_iter() {
        let flags = TestU8::F2 | TestU8::F4 | TestU8::F6;
        let mut i = flags.iter();
        assert_eq!(i.next(), Some(&EnumVariant::new("F2", Some(TestU8::F2))));
        assert_eq!(i.next(), Some(&EnumVariant::new("F4", Some(TestU8::F4))));
        assert_eq!(i.next(), Some(&EnumVariant::new("F6", Some(TestU8::F6))));
        assert_eq!(i.next(), None);
        assert_eq!(flags.len(), 3);
    }

    #[test]
    fn test_flag_len() {
        assert_eq!((TestU8::F2 | TestU8::F4 | TestU8::F6).len(), 3);
        assert_eq!(Flags::<TestEmpty>::empty().len(), 0);
        assert_eq!(Flags::<TestEmpty>::full().len(), 0);
        assert_eq!(Flags::<TestU8>::empty().len(), 0);
        assert_eq!(Flags::<TestU8>::full().len(), 8);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_flag_serde() {
        let flags = TestU8::F2 | TestU8::F4 | TestU8::F6;
        let serialized = serde_json::to_string(&flags).unwrap();
        assert_eq!(&serialized, "42"); // 101010 as Dec
        let deserialized = serde_json::from_str::<Flags<TestU8>>(&serialized).unwrap();
        assert_eq!(flags, deserialized);
    }
}
