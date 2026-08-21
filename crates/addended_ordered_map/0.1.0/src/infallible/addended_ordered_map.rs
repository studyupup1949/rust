/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::collections::btree_map;
use core::{convert::Infallible, fmt, ops::RangeBounds};

use super::{AddendableKey, SizedValue};
use crate::fallible::AddendedOrderedMapFallible;
use crate::FindSettings;

/// An ordered mapping with addended lookups per key.
///
/// Each lookup can be either be an exact lookup (like a classic mapping), or
/// it can be done against each key plus an offset (addend). This behaviour
/// allows each key to be its own kind of "ranged" value, where the size of the
/// range is given by the associated value for each key.
///
/// The range for each key is a half open one, where the start is closed but
/// the end one is open (inclusive, exclusive range).
///
/// Keys are required to implement the [`Ord`] and [`AddendableKey`] traits,
/// while values need to implement the [`SizedValue`] trait.
///
/// Contrary to common mappings, this type requires a third generic parameter,
/// the `SIZE`.
/// This type parameter is used as an intermediary type for the range
/// calculation for a key, allowing this size type to be different from the key
/// and value types.
///
/// Ranges are not set in stone, they are computed based on the corresponding
/// value for a given key using the [`SizedValue`] trait each time the range is
/// needed. This allows sizes (and thus the end of the range) to be modified
/// _after_ the key/value pair has been inserted into the mapping. This can be
/// useful if there's not certainty on where the range of a given key ends at
/// construction time.
///
/// # Examples
///
/// ```
/// use addended_ordered_map::{AddendedOrderedMap, FindSettings};
///
/// let settings = FindSettings::new(true);
/// let mut map = AddendedOrderedMap::new();
///
/// // Insert a key with size 4.
/// // In other words, contains the range `[0x1000, 0x1004)` (closed, open).
/// // For plain types like integers their own value is considered the size too,
/// // more complex types need to implement the `SizedValue` trait.
/// let (value, newly_inserted) = map.find_mut_or_insert_with(0x1000, settings, || 4);
/// assert_eq!(value, &4);
/// assert_eq!(newly_inserted, true);
///
/// // This does not insert the key 0x1001 because it overlaps with the range
/// // of the previous key.
/// let (value, newly_inserted) = map.find_mut_or_insert_with(0x1001, settings, || 4);
/// assert_eq!(value, &4);
/// assert_eq!(newly_inserted, false);
///
/// assert_eq!(map.len(), 1);
///
/// assert_eq!(
///     map.find(&0x1000, settings),
///     Some((&0x1000, &4)),
/// );
///
/// // Ranged lookups work.
/// assert_eq!(
///     map.find(&0x1002, settings),
///     Some((&0x1000, &4)),
/// );
/// // We can also do exact key lookups if wanted.
/// assert_eq!(
///     map.find(&0x1002, FindSettings::new(false)),
///     None,
/// );
///
/// // This is outside the range of the only key.
/// assert_eq!(
///     map.find(&0x1004, settings),
///     None,
/// );
/// ```
///
/// Custom types.
///
/// ```
/// use addended_ordered_map::{AddendedOrderedMap, AddendableKey, FindSettings, SizedValue};
///
/// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// struct TestKey(u32);
/// #[derive(Debug, PartialEq)]
/// struct TestValue(&'static str, u32);
///
/// impl AddendableKey<u32> for TestKey {
///     fn add_size(&self, size: &u32) -> Self {
///         Self(self.0 + *size)
///     }
/// }
/// impl SizedValue<u32> for TestValue {
///     fn size(&self) -> u32 {
///         self.1
///     }
/// }
///
/// let mut map: AddendedOrderedMap<TestKey, TestValue, u32> = AddendedOrderedMap::new();
///
/// let settings = FindSettings::new(true);
///
/// map.find_mut_or_insert_with(TestKey(0x1000), settings, || TestValue("value1", 8));
/// map.find_mut_or_insert_with(TestKey(0x1007), settings, || TestValue("value1", 8));
///
/// assert_eq!(
///     map.find(&TestKey(0x1000), settings),
///     Some((&TestKey(0x1000), &TestValue("value1", 8))),
/// );
///
/// assert_eq!(
///     map.find(&TestKey(0x1004), settings),
///     Some((&TestKey(0x1000), &TestValue("value1", 8))),
/// );
///
/// assert_eq!(
///     map.find(&TestKey(0x1004), FindSettings::new(false)),
///     None,
/// );
/// ```
///
/// [`Ord`]: core::cmp::Ord
/// [`AddendableKey`]: crate::infallible::AddendableKey
/// [`SizedValue`]: crate::infallible::SizedValue
pub struct AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    inner: AddendedOrderedMapFallible<K, V, SIZE, Infallible>,
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    /// Creates a new `AddendedOrderedMap`.
    pub fn new() -> Self {
        Self {
            inner: AddendedOrderedMapFallible::new(),
        }
    }

    /// Returns the amount of elements in this mapping.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Checks if the mapping is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    /// Find a key/value pair within the range of the given key parameter.
    ///
    /// Use [`FindSettings`] with `allow_addend` being `true` to allow ranged
    /// lookups.
    /// This allows checking if the given key is within the range of any key of
    /// the mapping plus the addend given by the value associated to that key.
    ///
    /// Use [`FindSettings`] with `allow_addend=false` to do exact lookups.
    ///
    /// For a mutable variant, see [`find_mut`].
    ///
    /// [`FindSettings`]: crate::FindSettings
    /// [`find_mut`]: AddendedOrderedMap::find_mut
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find(&self, key: &K, settings: FindSettings) -> Option<(&K, &V)> {
        self.inner
            .find(key, settings)
            .expect("Infallible operation")
    }

    /// Find a key within the range of the given key parameter.
    ///
    /// Use [`FindSettings`] with `allow_addend` being `true` to allow ranged
    /// lookups.
    /// This allows checking if the given key parameter is within the range of
    /// any key of the mapping plus the addend given by the value associated to
    /// that key.
    ///
    /// Use [`FindSettings`] with `allow_addend=false` to do exact lookups.
    ///
    /// This method is equivalent to `.find().map(|x| x.0)`.
    ///
    /// There's no mutable variant because keys are not mutable.
    ///
    /// [`FindSettings`]: crate::FindSettings
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_key(&self, key: &K, settings: FindSettings) -> Option<&K> {
        self.inner
            .find_key(key, settings)
            .expect("Infallible operation")
    }

    /// Find a value associated to the range of the given key parameter.
    ///
    /// Use [`FindSettings`] with `allow_addend` being `true` to allow ranged
    /// lookups.
    /// This allows checking if the given key parameter is within the range of
    /// any key of the mapping plus the addend given by the value associated to
    /// that key.
    ///
    /// Use [`FindSettings`] with `allow_addend=false` to do exact lookups.
    ///
    /// This method is equivalent to `.find().map(|x| x.1)`.
    ///
    /// For a mutable variant, see [`find_value_mut`].
    ///
    /// [`FindSettings`]: crate::FindSettings
    /// [`find_value_mut`]: AddendedOrderedMap::find_value_mut
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_value(&self, key: &K, settings: FindSettings) -> Option<&V> {
        self.inner
            .find_value(key, settings)
            .expect("Infallible operation")
    }

    /// Find a mutable key/value pair within the range of the given key
    /// parameter.
    ///
    /// Note only the value is mutable.
    ///
    /// Use [`FindSettings`] with `allow_addend` being `true` to allow ranged
    /// lookups.
    /// This allows checking if the given key is within the range of any key of
    /// the mapping plus the addend given by the value associated to that key.
    ///
    /// Use [`FindSettings`] with `allow_addend=false` to do exact lookups.
    ///
    /// For a non-mutable variant, see [`find`].
    ///
    /// [`FindSettings`]: crate::FindSettings
    /// [`find`]: AddendedOrderedMap::find
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_mut(&mut self, key: &K, settings: FindSettings) -> Option<(&K, &mut V)> {
        self.inner
            .find_mut(key, settings)
            .expect("Infallible operation")
    }

    /// Find a mutable value associated to the range of the given key parameter.
    ///
    /// Use [`FindSettings`] with `allow_addend` being `true` to allow ranged
    /// lookups.
    /// This allows checking if the given key parameter is within the range of
    /// any key of the mapping plus the addend given by the value associated to
    /// that key.
    ///
    /// Use [`FindSettings`] with `allow_addend=false` to do exact lookups.
    ///
    /// This method is equivalent to `.find_mut().map(|x| x.1)`.
    ///
    /// For a non-mutable variant, see [`find_value`].
    ///
    /// [`FindSettings`]: crate::FindSettings
    /// [`find_value`]: AddendedOrderedMap::find_value
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_value_mut(&mut self, key: &K, settings: FindSettings) -> Option<&mut V> {
        self.inner
            .find_value_mut(key, settings)
            .expect("Infallible operation")
    }

    /// Lookups for the key that is at the left, or less (or equal) than, the
    /// given key parameter.
    ///
    /// Returns either the first key that is at the left of the key parameter,
    /// without considering ranges.
    ///
    /// Equality of keys is considered if the parameter `inclusive` is `true`.
    ///
    /// For a mutable variant see [`find_left_of_mut`].
    ///
    /// # Examples
    ///
    /// ```
    /// use addended_ordered_map::{AddendedOrderedMap, FindSettings};
    ///
    /// let mut map: AddendedOrderedMap<u32, Option<u32>, u32> = AddendedOrderedMap::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x100C, settings, || None);
    /// map.find_mut_or_insert_with(0x1000, settings, || Some(4));
    /// map.find_mut_or_insert_with(0x1004, settings, || Some(4));
    ///
    /// // The exact key does exist, so the inclusive range returns the key itself.
    /// assert_eq!(
    ///     map.find_left_of(&0x1004, true),
    ///     Some((&0x1004, &Some(4))),
    /// );
    /// // Exclusive lookup returns the next lower key.
    /// assert_eq!(
    ///     map.find_left_of(&0x1004, false),
    ///     Some((&0x1000, &Some(4))),
    /// );
    /// // There's nothing at the left of (strict less than) this key, so it
    /// // returns `None`
    /// assert_eq!(
    ///     map.find_left_of(&0x1000, false),
    ///     None,
    /// );
    /// ```
    ///
    /// [`find_left_of_mut`]: AddendedOrderedMap::find_left_of_mut
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_left_of(&self, key: &K, inclusive: bool) -> Option<(&K, &V)> {
        self.inner.find_left_of(key, inclusive)
    }

    /// Lookups for the key that is at the right, or greater (or equal) than, the
    /// given key parameter.
    ///
    /// Returns either the first key that is at the right of the key parameter,
    /// without considering ranges.
    ///
    /// Equality of keys is considered if the parameter `inclusive` is `true`.
    ///
    /// For a mutable variant see [`find_right_of_mut`].
    ///
    /// # Examples
    ///
    /// ```
    /// use addended_ordered_map::{AddendedOrderedMap, FindSettings};
    ///
    /// let mut map: AddendedOrderedMap<u32, Option<u32>, u32> = AddendedOrderedMap::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x100C, settings, || None);
    /// map.find_mut_or_insert_with(0x1000, settings, || Some(4));
    /// map.find_mut_or_insert_with(0x1004, settings, || Some(4));
    ///
    /// // The exact key does exist, so the inclusive range returns the key itself.
    /// assert_eq!(
    ///     map.find_right_of(&0x1004, true),
    ///     Some((&0x1004, &Some(4))),
    /// );
    /// // Exclusive lookup returns the next greater key.
    /// assert_eq!(
    ///     map.find_right_of(&0x1004, false),
    ///     Some((&0x100C, &None)),
    /// );
    /// // There's nothing at the left of (strict less than) this key, so it
    /// // returns `None`
    /// assert_eq!(
    ///     map.find_right_of(&0x100C, false),
    ///     None,
    /// );
    /// ```
    ///
    /// [`find_right_of_mut`]: AddendedOrderedMap::find_right_of_mut
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_right_of(&self, key: &K, inclusive: bool) -> Option<(&K, &V)> {
        self.inner.find_right_of(key, inclusive)
    }

    /// Lookups for the key that is at the left, or less (or equal) than, the
    /// given key parameter.
    ///
    /// Returns either the first key that is at the left of the key parameter,
    /// without considering ranges.
    ///
    /// Equality of keys is considered if the parameter `inclusive` is `true`.
    ///
    /// For a non mutable variant see [`find_left_of`].
    ///
    /// # Examples
    ///
    /// ```
    /// use addended_ordered_map::{AddendedOrderedMap, FindSettings};
    ///
    /// let mut map: AddendedOrderedMap<u32, Option<u32>, u32> = AddendedOrderedMap::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x100C, settings, || None);
    /// map.find_mut_or_insert_with(0x1000, settings, || Some(4));
    /// map.find_mut_or_insert_with(0x1004, settings, || Some(4));
    ///
    /// // The exact key does exist, so the inclusive range returns the key itself.
    /// assert_eq!(
    ///     map.find_left_of_mut(&0x1004, true),
    ///     Some((&0x1004, &mut Some(4))),
    /// );
    /// // Exclusive lookup returns the next lower key.
    /// assert_eq!(
    ///     map.find_left_of_mut(&0x1004, false),
    ///     Some((&0x1000, &mut Some(4))),
    /// );
    /// // There's nothing at the left of (strict less than) this key, so it
    /// // returns `None`
    /// assert_eq!(
    ///     map.find_left_of_mut(&0x1000, false),
    ///     None,
    /// );
    /// ```
    ///
    /// [`find_left_of`]: AddendedOrderedMap::find_left_of
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_left_of_mut(&mut self, key: &K, inclusive: bool) -> Option<(&K, &mut V)> {
        self.inner.find_left_of_mut(key, inclusive)
    }

    /// Lookups for the key that is at the right, or greater (or equal) than, the
    /// given key parameter.
    ///
    /// Returns either the first key that is at the right of the key parameter,
    /// without considering ranges.
    ///
    /// Equality of keys is considered if the parameter `inclusive` is `true`.
    ///
    /// For a non utable variant see [`find_right_of`].
    ///
    /// # Examples
    ///
    /// ```
    /// use addended_ordered_map::{AddendedOrderedMap, FindSettings};
    ///
    /// let mut map: AddendedOrderedMap<u32, Option<u32>, u32> = AddendedOrderedMap::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x100C, settings, || None);
    /// map.find_mut_or_insert_with(0x1000, settings, || Some(4));
    /// map.find_mut_or_insert_with(0x1004, settings, || Some(4));
    ///
    /// // The exact key does exist, so the inclusive range returns the key itself.
    /// assert_eq!(
    ///     map.find_right_of_mut(&0x1004, true),
    ///     Some((&0x1004, &mut Some(4))),
    /// );
    /// // Exclusive lookup returns the next greater key.
    /// assert_eq!(
    ///     map.find_right_of_mut(&0x1004, false),
    ///     Some((&0x100C, &mut None)),
    /// );
    /// // There's nothing at the left of (strict less than) this key, so it
    /// // returns `None`
    /// assert_eq!(
    ///     map.find_right_of_mut(&0x100C, false),
    ///     None,
    /// );
    /// ```
    ///
    /// [`find_right_of`]: AddendedOrderedMap::find_right_of
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_right_of_mut(&mut self, key: &K, inclusive: bool) -> Option<(&K, &mut V)> {
        self.inner.find_right_of_mut(key, inclusive)
    }
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    /// Either find the given key or insert a value associated to the key.
    ///
    /// Tries to search up for the given key (either ranged or not depending on
    /// the settings) and returns the associated value.
    ///
    /// If the key isn't found, the key is inserted with the associated value
    /// returned from the `default` function parameter.
    ///
    /// If the key is found, then `default` is not called.
    ///
    /// The returned bool is `true` if a new value was inserted.
    /// `false` if a previously existing key was returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use addended_ordered_map::{AddendedOrderedMap, FindSettings};
    ///
    /// let mut map: AddendedOrderedMap<u32, u32, u32> = AddendedOrderedMap::new();
    /// let settings = FindSettings::new(true);
    ///
    /// let (v, newly_inserted) = map.find_mut_or_insert_with(0x1000, settings, || 8);
    ///
    /// assert_eq!(v, &8);
    /// assert_eq!(newly_inserted, true);
    ///
    /// let (v, newly_inserted) = map.find_mut_or_insert_with(0x1004, settings, || 4);
    ///
    /// // The key already existed (in the [0x1000, 0x1008) range), so the same
    /// // old value was returned.
    /// assert_eq!(v, &8);
    /// assert_eq!(newly_inserted, false);
    /// ```
    // TODO: return the key too.
    #[doc(alias = "insert")]
    pub fn find_mut_or_insert_with<F>(
        &mut self,
        key: K,
        settings: FindSettings,
        default: F,
    ) -> (&mut V, bool)
    where
        F: FnOnce() -> V,
    {
        self.inner
            .find_mut_or_insert_with(key, settings, || Ok(default()))
            .expect("Infallible operation")
    }

    // TODO: plain insert. replace_or_insert?
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    /// Checks if a key is contained on the mapping, ignoring ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use addended_ordered_map::{AddendedOrderedMap, FindSettings};
    ///
    /// let mut map = AddendedOrderedMap::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x1000, settings, || 4);
    ///
    /// assert_eq!(
    ///     map.contains_key_exact(&0x1000),
    ///     true,
    /// );
    /// // This key is within the range of the 0x1000 key, but still returns false.
    /// assert_eq!(
    ///     map.contains_key_exact(&0x1002),
    ///     false,
    /// );
    /// ```
    pub fn contains_key_exact(&self, key: &K) -> bool {
        self.inner.contains_key_exact(key)
    }

    /// Remove and return a pair if the exact key is present on the mapping,
    /// ignoring ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use addended_ordered_map::{AddendedOrderedMap, FindSettings};
    ///
    /// let mut map = AddendedOrderedMap::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x1000, settings, || 4);
    ///
    /// // This key is within the range of the 0x1000 key, but returns None.
    /// assert_eq!(
    ///     map.pop_exact(&0x1002),
    ///     None,
    /// );
    /// assert_eq!(
    ///     map.pop_exact(&0x1000),
    ///     Some((0x1000, 4)),
    /// );
    /// ```
    pub fn pop_exact(&mut self, key: &K) -> Option<(K, V)> {
        self.inner.pop_exact(key)
    }

    /// Removes all the elements from the mapping.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)`
    /// returns `false`. The elements are visited in ascending key order.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.inner.retain(f);
    }
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    /// Gets an iterator over the entries of the map, sorted by key.
    pub fn iter(&self) -> btree_map::Iter<'_, K, V> {
        self.inner.iter()
    }

    /// Gets a mutable iterator over the entries of the map, sorted by key.
    pub fn iter_mut(&mut self) -> btree_map::IterMut<'_, K, V> {
        self.inner.iter_mut()
    }

    /// Gets an iterator over the specific range of the map, sorted by key.
    pub fn range<R>(&self, range: R) -> btree_map::Range<'_, K, V>
    where
        R: RangeBounds<K>,
    {
        self.inner.range(range)
    }

    /// Gets a mutable iterator over the specific range of the map, sorted by key.
    pub fn range_mut<R>(&mut self, range: R) -> btree_map::RangeMut<'_, K, V>
    where
        R: RangeBounds<K>,
    {
        self.inner.range_mut(range)
    }

    /// Creates an iterator that visits elements (key-value pairs) in the
    /// specified range in ascending key order and uses a closure to determine
    /// if an element should be removed.
    ///
    /// If the closure returns true, the element is removed from the map and
    /// yielded. If the closure returns false, or panics, the element remains
    /// in the map and will not be yielded.
    ///
    /// The iterator also lets you mutate the value of each element in the
    /// closure, regardless of whether you choose to keep or remove it.
    ///
    /// If the returned ExtractIf is not exhausted, e.g. because it is dropped
    /// without iterating or the iteration short-circuits, then the remaining
    /// elements will be retained.
    #[cfg(feature = "extract_if")]
    #[clippy::msrv = "1.91"]
    pub fn extract_if<F, R>(&mut self, range: R, pred: F) -> btree_map::ExtractIf<'_, K, V, R, F>
    where
        R: RangeBounds<K>,
        F: FnMut(&K, &mut V) -> bool,
    {
        self.inner.extract_if(range, pred)
    }

    /// Gets an iterator over the keys of the map, in sorted order.
    pub fn keys(&self) -> btree_map::Keys<'_, K, V> {
        self.inner.keys()
    }

    /// Gets an iterator over the values of the map, in order by key.
    pub fn values(&self) -> btree_map::Values<'_, K, V> {
        self.inner.values()
    }

    /// Gets a mutable iterator over the values of the map, in order by key.
    pub fn values_mut(&mut self) -> btree_map::ValuesMut<'_, K, V> {
        self.inner.values_mut()
    }

    /// Creates a consuming iterator visiting all the keys, in sorted order.
    pub fn into_keys(self) -> btree_map::IntoKeys<K, V> {
        self.inner.into_keys()
    }

    /// Creates a consuming iterator visiting all the values, in order by key.
    pub fn into_values(self) -> btree_map::IntoValues<K, V> {
        self.inner.into_values()
    }
}

impl<K, V, SIZE> fmt::Debug for AddendedOrderedMap<K, V, SIZE>
where
    K: fmt::Debug + Ord + AddendableKey<SIZE>,
    V: fmt::Debug + SizedValue<SIZE>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Manually implement Debug to hide the `inner` indirection
        write!(f, "AddendedOrderedMap {:?}", self.inner)
    }
}

impl<K, V, SIZE> Clone for AddendedOrderedMap<K, V, SIZE>
where
    K: Clone + Ord + AddendableKey<SIZE>,
    V: Clone + SizedValue<SIZE>,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V, SIZE> core::hash::Hash for AddendedOrderedMap<K, V, SIZE>
where
    K: core::hash::Hash + Ord + AddendableKey<SIZE>,
    V: core::hash::Hash + SizedValue<SIZE>,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<K, V, SIZE> PartialEq for AddendedOrderedMap<K, V, SIZE>
where
    K: PartialEq + Ord + AddendableKey<SIZE>,
    V: PartialEq + SizedValue<SIZE>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<K, V, SIZE> Eq for AddendedOrderedMap<K, V, SIZE>
where
    K: Eq + Ord + AddendableKey<SIZE>,
    V: Eq + SizedValue<SIZE>,
{
}

impl<K, V, SIZE> PartialOrd for AddendedOrderedMap<K, V, SIZE>
where
    K: PartialOrd + Ord + AddendableKey<SIZE>,
    V: PartialOrd + SizedValue<SIZE>,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<K, V, SIZE> Ord for AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: Ord + SizedValue<SIZE>,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<K, V, SIZE> Default for AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V, SIZE> IntoIterator for &'a AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    type Item = (&'a K, &'a V);
    type IntoIter = btree_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, SIZE> IntoIterator for &'a mut AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = btree_map::IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, SIZE> IntoIterator for AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + AddendableKey<SIZE>,
    V: SizedValue<SIZE>,
{
    type Item = (K, V);
    type IntoIter = btree_map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_bounds() {
        let mut map: AddendedOrderedMap<u32, Option<u32>, u32> = AddendedOrderedMap::new();

        map.find_mut_or_insert_with(0x100C, FindSettings::new(true), || None);
        map.find_mut_or_insert_with(0x1000, FindSettings::new(true), || Some(4));
        map.find_mut_or_insert_with(0x1004, FindSettings::new(true), || Some(4));

        assert_eq!(
            map.find(&0x1000, FindSettings::new(true)),
            Some((&0x1000, &Some(4))),
        );

        assert_eq!(
            map.find(&0x1002, FindSettings::new(true)),
            Some((&0x1000, &Some(4))),
        );

        assert_eq!(map.find(&0x0F00, FindSettings::new(true)), None);

        assert_eq!(map.find(&0x2000, FindSettings::new(true)), None);

        assert_eq!(map.find(&0x1002, FindSettings::new(false)), None);

        assert_eq!(map.find(&0x1008, FindSettings::new(true)), None);
    }

    #[test]
    fn check_left_right() {
        let mut map: AddendedOrderedMap<u32, Option<u32>, u32> = AddendedOrderedMap::new();

        map.find_mut_or_insert_with(0x100C, FindSettings::new(true), || None);
        map.find_mut_or_insert_with(0x1000, FindSettings::new(true), || Some(4));
        map.find_mut_or_insert_with(0x1004, FindSettings::new(true), || Some(4));

        assert_eq!(map.find_left_of(&0x1004, true), Some((&0x1004, &Some(4))),);
        assert_eq!(map.find_left_of(&0x1004, false), Some((&0x1000, &Some(4))),);
        assert_eq!(map.find_right_of(&0x1004, true), Some((&0x1004, &Some(4))),);
        assert_eq!(map.find_right_of(&0x1004, false), Some((&0x100C, &None)),);

        assert_eq!(
            map.find_left_of(&0x1004, true),
            map.find_right_of(&0x1004, true),
        );
    }

    #[test]
    fn check_custom_type() {
        // Debug required for assert_eq!
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct TestKey(u32);
        #[derive(Debug, PartialEq)]
        struct TestValue(&'static str, u32);

        impl AddendableKey<u32> for TestKey {
            fn add_size(&self, size: &u32) -> Self {
                Self(self.0 + *size)
            }
        }
        impl SizedValue<u32> for TestValue {
            fn size(&self) -> u32 {
                self.1
            }
        }

        let mut map: AddendedOrderedMap<TestKey, TestValue, u32> = AddendedOrderedMap::new();

        let settings = FindSettings::new(true);

        map.find_mut_or_insert_with(TestKey(0x1000), settings, || TestValue("value1", 8));
        map.find_mut_or_insert_with(TestKey(0x1007), settings, || TestValue("value1", 8));

        assert_eq!(
            map.find(&TestKey(0x1000), settings),
            Some((&TestKey(0x1000), &TestValue("value1", 8))),
        );

        assert_eq!(
            map.find(&TestKey(0x1004), settings),
            Some((&TestKey(0x1000), &TestValue("value1", 8))),
        );

        assert_eq!(map.find(&TestKey(0x1004), FindSettings::new(false)), None,);
    }
}
