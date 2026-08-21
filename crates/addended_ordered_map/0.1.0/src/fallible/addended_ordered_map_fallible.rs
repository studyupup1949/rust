/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::collections::btree_map::{self, BTreeMap};
use core::{
    fmt,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

#[cfg(not(feature = "nightly"))]
use ::polonius_the_crab::prelude::*;

use super::{AddendableKeyFallible, SizedValueFallible};
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
/// Keys are required to implement the [`Ord`] and [`AddendableKeyFallible`]
/// traits, while values need to implement the [`SizedValueFallible`] trait.
///
/// Contrary to common mappings, this type requires a third generic parameter,
/// the `SIZE`.
/// This type parameter is used as an intermediary type for the range
/// calculation for a key, allowing this size type to be different from the key
/// and value types.
///
/// Ranges are not set in stone, they are computed based on the corresponding
/// value for a given key using the [`SizedValueFallible`] trait each time the
/// range is needed. This allows sizes (and thus the end of the range) to be
/// modified _after_ the key/value pair has been inserted into the mapping.
/// This can be useful if there's not certainty on where the range of a given
/// key ends at construction time.
///
/// The forth generic type parameter `E` allow fallible size and key-addition
/// operations, so this map works with types whose addend or size computations
/// may fail.
///
/// # Examples
///
/// ```
/// use addended_ordered_map::{fallible::AddendedOrderedMapFallible, FindSettings};
///
/// let settings = FindSettings::new(true);
/// let mut map = AddendedOrderedMapFallible::new();
///
/// // Insert a key with size 4.
/// // In other words, contains the range `[0x1000, 0x1004)` (closed, open).
/// // For plain types like integers their own value is considered the size too,
/// // more complex types need to implement the `SizedValue` trait.
/// let (value, newly_inserted) = map.find_mut_or_insert_with(0x1000, settings, || Ok(4)).unwrap();
/// assert_eq!(value, &4);
/// assert_eq!(newly_inserted, true);
///
/// // This does not insert the key 0x1001 because it overlaps with the range
/// // of the previous key.
/// let (value, newly_inserted) = map.find_mut_or_insert_with(0x1001, settings, || Ok(4)).unwrap();
/// assert_eq!(value, &4);
/// assert_eq!(newly_inserted, false);
///
/// assert_eq!(map.len(), 1);
///
/// assert_eq!(
///     map.find(&0x1000, settings),
///     Ok(Some((&0x1000, &4))),
/// );
///
/// // Ranged lookups work.
/// assert_eq!(
///     map.find(&0x1002, settings),
///     Ok(Some((&0x1000, &4))),
/// );
/// // We can also do exact key lookups if wanted.
/// assert_eq!(
///     map.find(&0x1002, FindSettings::new(false)),
///     Ok(None),
/// );
///
/// // This is outside the range of the only key.
/// assert_eq!(
///     map.find(&0x1004, settings),
///     Ok(None),
/// );
///
/// let old_size = map.len();
///
/// // The callback may return Err, which produces the function to return Err
/// // too.
/// let ret = map.find_mut_or_insert_with(0x2000, settings, || Err(()));
/// assert_eq!(
///     ret,
///     Err(()),
/// );
///
/// // No value is inserted if an insertion fail happens.
/// assert_eq!(map.len(), old_size);
/// ```
///
/// Custom types.
///
/// ```
/// use addended_ordered_map::{
///     fallible::{AddendedOrderedMapFallible, AddendableKeyFallible, SizedValueFallible},
///     FindSettings,
/// };
///
/// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// struct TestKey(u32);
/// #[derive(Debug, PartialEq)]
/// struct TestValue(&'static str, u32);
/// #[derive(Debug, PartialEq)]
/// struct Bad;
///
/// impl AddendableKeyFallible<u32, Bad> for TestKey {
///     fn add_size(&self, size: &u32) -> Result<Self, Bad> {
///         Ok(Self(self.0 + *size))
///     }
/// }
/// impl SizedValueFallible<u32, Bad> for TestValue {
///     fn size(&self) -> Result<u32, Bad> {
///         Ok(self.1)
///     }
/// }
///
/// let mut map: AddendedOrderedMapFallible<TestKey, TestValue, u32, Bad> = AddendedOrderedMapFallible::new();
///
/// let settings = FindSettings::new(true);
///
/// map.find_mut_or_insert_with(TestKey(0x1000), settings, || Ok(TestValue("value1", 8)));
/// map.find_mut_or_insert_with(TestKey(0x1007), settings, || Ok(TestValue("value1", 8)));
///
/// assert_eq!(
///     map.find(&TestKey(0x1000), settings),
///     Ok(Some((&TestKey(0x1000), &TestValue("value1", 8)))),
/// );
///
/// assert_eq!(
///     map.find(&TestKey(0x1004), settings),
///     Ok(Some((&TestKey(0x1000), &TestValue("value1", 8)))),
/// );
///
/// assert_eq!(
///     map.find(&TestKey(0x1004), FindSettings::new(false)),
///     Ok(None),
/// );
/// ```
///
/// [`Ord`]: core::cmp::Ord
/// [`AddendableKeyFallible`]: crate::fallible::AddendableKeyFallible
/// [`SizedValueFallible`]: crate::fallible::SizedValueFallible
pub struct AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
{
    inner: BTreeMap<K, V>,
    phantom: PhantomData<(SIZE, E)>,
}

impl<K, V, SIZE, E> AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
{
    /// Creates a new `AddendedOrderedMapFallible`.
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
            phantom: PhantomData,
        }
    }

    /// Returns the amount of elements in this mapping.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Checks if the mapping is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K, V, SIZE, E> AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
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
    /// If the underlying `add_size` or `size` operation fails, the error is
    /// returned directly and no lookup result is produced.
    ///
    /// For a mutable variant, see [`find_mut`].
    ///
    /// [`FindSettings`]: crate::FindSettings
    /// [`find_mut`]: AddendedOrderedMapFallible::find_mut
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find(&self, key: &K, settings: FindSettings) -> Result<Option<(&K, &V)>, E> {
        let mut range = self.inner.range(..=key);

        let ret = if let Some((other_key, v)) = range.next_back() {
            if other_key == key
                || (settings.allow_addend && key < &other_key.add_size(&v.size()?)?)
            {
                Some((other_key, v))
            } else {
                None
            }
        } else {
            None
        };

        Ok(ret)
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
    pub fn find_key(&self, key: &K, settings: FindSettings) -> Result<Option<&K>, E> {
        self.find(key, settings).map(|x| x.map(|y| y.0))
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
    /// [`find_value_mut`]: AddendedOrderedMapFallible::find_value_mut
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_value(&self, key: &K, settings: FindSettings) -> Result<Option<&V>, E> {
        self.find(key, settings).map(|x| x.map(|y| y.1))
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
    /// [`find`]: AddendedOrderedMapFallible::find
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_mut(&mut self, key: &K, settings: FindSettings) -> Result<Option<(&K, &mut V)>, E> {
        let mut range = self.inner.range_mut(..=key);

        let ret = if let Some((other_key, v)) = range.next_back() {
            if other_key == key
                || (settings.allow_addend && key < &other_key.add_size(&v.size()?)?)
            {
                Some((other_key, v))
            } else {
                None
            }
        } else {
            None
        };

        Ok(ret)
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
    /// [`find_value`]: AddendedOrderedMapFallible::find_value
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_value_mut(&mut self, key: &K, settings: FindSettings) -> Result<Option<&mut V>, E> {
        self.find_mut(key, settings).map(|x| x.map(|y| y.1))
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
    /// use addended_ordered_map::{fallible::AddendedOrderedMapFallible, FindSettings};
    ///
    /// let mut map: AddendedOrderedMapFallible<u32, Option<u32>, u32, ()> = AddendedOrderedMapFallible::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x100C, settings, || Ok(None));
    /// map.find_mut_or_insert_with(0x1000, settings, || Ok(Some(4)));
    /// map.find_mut_or_insert_with(0x1004, settings, || Ok(Some(4)));
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
    /// [`find_left_of_mut`]: AddendedOrderedMapFallible::find_left_of_mut
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_left_of(&self, key: &K, inclusive: bool) -> Option<(&K, &V)> {
        let start = Bound::Unbounded;
        let end = if inclusive {
            Bound::Included(key)
        } else {
            Bound::Excluded(key)
        };

        self.inner.range((start, end)).next_back()
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
    /// use addended_ordered_map::{fallible::AddendedOrderedMapFallible, FindSettings};
    ///
    /// let mut map: AddendedOrderedMapFallible<u32, Option<u32>, u32, ()> = AddendedOrderedMapFallible::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x100C, settings, || Ok(None));
    /// map.find_mut_or_insert_with(0x1000, settings, || Ok(Some(4)));
    /// map.find_mut_or_insert_with(0x1004, settings, || Ok(Some(4)));
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
    /// [`find_right_of_mut`]: AddendedOrderedMapFallible::find_right_of_mut
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_right_of(&self, key: &K, inclusive: bool) -> Option<(&K, &V)> {
        let start = if inclusive {
            Bound::Included(key)
        } else {
            Bound::Excluded(key)
        };
        let end = Bound::Unbounded;

        self.inner.range((start, end)).next()
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
    /// use addended_ordered_map::{fallible::AddendedOrderedMapFallible, FindSettings};
    ///
    /// let mut map: AddendedOrderedMapFallible<u32, Option<u32>, u32, ()> = AddendedOrderedMapFallible::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x100C, settings, || Ok(None));
    /// map.find_mut_or_insert_with(0x1000, settings, || Ok(Some(4)));
    /// map.find_mut_or_insert_with(0x1004, settings, || Ok(Some(4)));
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
    /// [`find_left_of`]: AddendedOrderedMapFallible::find_left_of
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_left_of_mut(&mut self, key: &K, inclusive: bool) -> Option<(&K, &mut V)> {
        let start = Bound::Unbounded;
        let end = if inclusive {
            Bound::Included(key)
        } else {
            Bound::Excluded(key)
        };

        self.inner.range_mut((start, end)).next_back()
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
    /// use addended_ordered_map::{fallible::AddendedOrderedMapFallible, FindSettings};
    ///
    /// let mut map: AddendedOrderedMapFallible<u32, Option<u32>, u32, ()> = AddendedOrderedMapFallible::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x100C, settings, || Ok(None));
    /// map.find_mut_or_insert_with(0x1000, settings, || Ok(Some(4)));
    /// map.find_mut_or_insert_with(0x1004, settings, || Ok(Some(4)));
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
    /// [`find_right_of`]: AddendedOrderedMapFallible::find_right_of
    #[must_use = "This is a lookup function, there are no side-effects on the mapping."]
    pub fn find_right_of_mut(&mut self, key: &K, inclusive: bool) -> Option<(&K, &mut V)> {
        let start = if inclusive {
            Bound::Included(key)
        } else {
            Bound::Excluded(key)
        };
        let end = Bound::Unbounded;

        self.inner.range_mut((start, end)).next()
    }
}

#[cfg(not(feature = "nightly"))]
fn add_impl<'slf, K, V, SIZE, E, F>(
    mut slf: &'slf mut AddendedOrderedMapFallible<K, V, SIZE, E>,
    key: K,
    settings: FindSettings,
    default: F,
) -> Result<(&'slf mut V, bool), E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
    F: FnOnce() -> Result<V, E>,
{
    // TODO: get rid of the polonius stuff when the new borrow checker has been released.

    polonius!(|slf| -> Result<(&'polonius mut V, bool), E> {
        let ret = match slf.find_mut(&key, settings) {
            Ok(r) => r,
            Err(e) => {
                polonius_return!(Err(e))
            }
        };
        if let Some((_k, v)) = ret {
            polonius_return!(Ok((v, false)));
        }
    });

    let v = default()?;
    let entry = slf.inner.entry(key);

    let newly_created = matches!(entry, btree_map::Entry::Vacant(_));
    Ok((entry.or_insert(v), newly_created))
}

#[cfg(feature = "nightly")]
fn add_impl<K, V, SIZE, E, F>(
    slf: &mut AddendedOrderedMapFallible<K, V, SIZE, E>,
    key: K,
    settings: FindSettings,
    default: F,
) -> Result<(&mut V, bool), E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
    F: FnOnce() -> Result<V, E>,
{
    let mut cursor = slf.inner.upper_bound_mut(Bound::Included(&key));

    let must_insert_new = if let Some((other_key, v)) = cursor.peek_prev() {
        if &key == other_key {
            false
        } else if !settings.allow_addend {
            true
        } else {
            key >= other_key.add_size(&v.size()?)?
        }
    } else {
        true
    };

    if must_insert_new {
        let v = default()?;
        cursor
            .insert_before(key, v)
            .expect("This should not be able to panic");
    }

    Ok((into_prev_and_next(cursor).0.unwrap().1, must_insert_new))
}

#[cfg(feature = "nightly")]
#[allow(clippy::type_complexity)]
fn into_prev_and_next<'a, K, V>(
    mut cursor: btree_map::CursorMut<'a, K, V>,
) -> (Option<(&'a K, &'a mut V)>, Option<(&'a K, &'a mut V)>) {
    let prev = cursor.peek_prev().map(map_lifetime_kv);
    let next = cursor.peek_next().map(map_lifetime_kv);

    (prev, next)
}

#[cfg(feature = "nightly")]
fn map_lifetime_kv<'old, 'new, K, V>((k, v): (&'old K, &'old mut V)) -> (&'new K, &'new mut V) {
    let ptr_k = k as *const K;
    let ptr_v = v as *mut V;
    // SAFETY: I don't think this is safe tbh
    unsafe { (&*ptr_k, &mut *ptr_v) }
}

impl<K, V, SIZE, E> AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
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
    /// use addended_ordered_map::{fallible::AddendedOrderedMapFallible, FindSettings};
    ///
    /// let mut map: AddendedOrderedMapFallible<u32, u32, u32, ()> = AddendedOrderedMapFallible::new();
    /// let settings = FindSettings::new(true);
    ///
    /// let (v, newly_inserted) = map.find_mut_or_insert_with(0x1000, settings, || Ok(8)).unwrap();
    ///
    /// assert_eq!(v, &8);
    /// assert_eq!(newly_inserted, true);
    ///
    /// let (v, newly_inserted) = map.find_mut_or_insert_with(0x1004, settings, || Ok(4)).unwrap();
    ///
    /// // The key already existed (in the [0x1000, 0x1008) range), so the same
    /// // old value was returned.
    /// assert_eq!(v, &8);
    /// assert_eq!(newly_inserted, false);
    /// ```
    pub fn find_mut_or_insert_with<F>(
        &mut self,
        key: K,
        settings: FindSettings,
        default: F,
    ) -> Result<(&mut V, bool), E>
    where
        F: FnOnce() -> Result<V, E>,
    {
        add_impl(self, key, settings, default)
    }
}

impl<K, V, SIZE, E> AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
{
    /// Checks if a key is contained on the mapping, ignoring ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use addended_ordered_map::{fallible::AddendedOrderedMapFallible, FindSettings};
    ///
    /// let mut map: AddendedOrderedMapFallible<_, _, _, ()> = AddendedOrderedMapFallible::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x1000, settings, || Ok(4));
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
        self.inner.contains_key(key)
    }

    /// Remove and return a pair if the exact key is present on the mapping,
    /// ignoring ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use addended_ordered_map::{fallible::AddendedOrderedMapFallible, FindSettings};
    ///
    /// let mut map: AddendedOrderedMapFallible<_, _, _, ()> = AddendedOrderedMapFallible::new();
    /// let settings = FindSettings::new(true);
    ///
    /// map.find_mut_or_insert_with(0x1000, settings, || Ok(4));
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
        self.inner.remove_entry(key)
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

impl<K, V, SIZE, E> AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
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

impl<K, V, SIZE, E> fmt::Debug for AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: fmt::Debug + Ord + AddendableKeyFallible<SIZE, E>,
    V: fmt::Debug + SizedValueFallible<SIZE, E>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Manually implement Debug to hide the `inner` indirection
        write!(f, "AddendedOrderedMapFallible {:?}", self.inner)
    }
}

impl<K, V, SIZE, E> Clone for AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Clone + Ord + AddendableKeyFallible<SIZE, E>,
    V: Clone + SizedValueFallible<SIZE, E>,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: self.phantom,
        }
    }
}

impl<K, V, SIZE, E> core::hash::Hash for AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: core::hash::Hash + Ord + AddendableKeyFallible<SIZE, E>,
    V: core::hash::Hash + SizedValueFallible<SIZE, E>,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
        self.phantom.hash(state);
    }
}

impl<K, V, SIZE, E> PartialEq for AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: PartialEq + Ord + AddendableKeyFallible<SIZE, E>,
    V: PartialEq + SizedValueFallible<SIZE, E>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner && self.phantom == other.phantom
    }
}

impl<K, V, SIZE, E> Eq for AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Eq + Ord + AddendableKeyFallible<SIZE, E>,
    V: Eq + SizedValueFallible<SIZE, E>,
{
}

impl<K, V, SIZE, E> PartialOrd for AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: PartialOrd + Ord + AddendableKeyFallible<SIZE, E>,
    V: PartialOrd + SizedValueFallible<SIZE, E>,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match self.inner.partial_cmp(&other.inner) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<K, V, SIZE, E> Ord for AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: Ord + SizedValueFallible<SIZE, E>,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner
            .cmp(&other.inner)
            .then_with(|| self.phantom.cmp(&other.phantom))
    }
}

impl<K, V, SIZE, E> Default for AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V, SIZE, E> IntoIterator for &'a AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
{
    type Item = (&'a K, &'a V);
    type IntoIter = btree_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, SIZE, E> IntoIterator for &'a mut AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = btree_map::IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, SIZE, E> IntoIterator for AddendedOrderedMapFallible<K, V, SIZE, E>
where
    K: Ord + AddendableKeyFallible<SIZE, E>,
    V: SizedValueFallible<SIZE, E>,
{
    type Item = (K, V);
    type IntoIter = btree_map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use super::*;

    #[test]
    fn check_bounds() {
        let mut map: AddendedOrderedMapFallible<u32, Option<u32>, u32, Infallible> =
            AddendedOrderedMapFallible::new();

        map.find_mut_or_insert_with(0x100C, FindSettings::new(true), || Ok(None))
            .unwrap();
        map.find_mut_or_insert_with(0x1000, FindSettings::new(true), || Ok(Some(4)))
            .unwrap();
        map.find_mut_or_insert_with(0x1004, FindSettings::new(true), || Ok(Some(4)))
            .unwrap();

        assert_eq!(
            Ok(Some((&0x1000, &Some(4)))),
            map.find(&0x1000, FindSettings::new(true)),
        );

        assert_eq!(
            Ok(Some((&0x1000, &Some(4)))),
            map.find(&0x1002, FindSettings::new(true)),
        );

        assert_eq!(Ok(None), map.find(&0x0F00, FindSettings::new(true)),);

        assert_eq!(Ok(None), map.find(&0x2000, FindSettings::new(true)),);

        assert_eq!(Ok(None), map.find(&0x1002, FindSettings::new(false)),);

        assert_eq!(Ok(None), map.find(&0x1008, FindSettings::new(true)),);
    }

    #[test]
    fn check_left_right() {
        let mut map: AddendedOrderedMapFallible<u32, Option<u32>, u32, Infallible> =
            AddendedOrderedMapFallible::new();

        map.find_mut_or_insert_with(0x100C, FindSettings::new(true), || Ok(None))
            .unwrap();
        map.find_mut_or_insert_with(0x1000, FindSettings::new(true), || Ok(Some(4)))
            .unwrap();
        map.find_mut_or_insert_with(0x1004, FindSettings::new(true), || Ok(Some(4)))
            .unwrap();

        assert_eq!(Some((&0x1004, &Some(4))), map.find_left_of(&0x1004, true),);
        assert_eq!(Some((&0x1000, &Some(4))), map.find_left_of(&0x1004, false),);

        assert_eq!(Some((&0x1004, &Some(4))), map.find_right_of(&0x1004, true),);
        assert_eq!(Some((&0x100C, &None)), map.find_right_of(&0x1004, false),);

        assert_eq!(
            map.find_left_of(&0x1004, true),
            map.find_right_of(&0x1004, true),
        );
    }

    #[test]
    fn small() {
        // Hold values from 0 to 100
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct Small(u8);

        #[derive(Debug, PartialEq, Eq)]
        struct Bad;

        impl Small {
            fn new(val: u8) -> Result<Self, Bad> {
                if val > 100 {
                    Err(Bad)
                } else {
                    Ok(Self(val))
                }
            }

            fn zero() -> Self {
                Self(0)
            }

            fn max() -> Self {
                Self(100)
            }
        }

        impl AddendableKeyFallible<Small, Bad> for Small {
            fn add_size(&self, size: &Small) -> Result<Self, Bad> {
                // Fails on overflowing 100
                let val = self.0.checked_add(size.0).ok_or(Bad)?;
                Self::new(val)
            }
        }

        impl SizedValueFallible<Small, Bad> for Small {
            fn size(&self) -> Result<Small, Bad> {
                Self::new(self.0)
            }
        }

        let mut map = AddendedOrderedMapFallible::new();
        let settings = FindSettings::new(true);

        let key = Small::new(50).unwrap();

        let ret = map.find_mut_or_insert_with(key, settings, || Small::new(20));
        assert_eq!(ret, Ok((&mut Small::new(20).unwrap(), true)),);

        let key = Small::new(50).unwrap();
        let key_plus_1 = Small::new(50 + 1).unwrap();

        // Not finding a value doesn't fail, it just returns Ok(None).
        let ret = map.find(&Small::zero(), settings);
        assert_eq!(ret, Ok(None),);
        let ret = map.find(&Small::max(), settings);
        assert_eq!(ret, Ok(None),);

        // Should not fail to find
        let ret = map.find_value_mut(&key, settings);
        let val = ret.unwrap();

        // Set to a value that will overflow if added with its key.
        let val = val.unwrap();
        *val = Small::new(90).unwrap();

        // Fails to do ranged lookups
        let ret = map.find(&key_plus_1, settings);
        assert_eq!(ret, Err(Bad),);

        // The only way to retrieve this value now is by doing exact lookups
        let ret = map.find(&key, settings);
        let big_value = Small::new(90).unwrap();
        assert_eq!(ret, Ok(Some((&key, &big_value))),);
    }
}
