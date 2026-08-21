use alloc::vec::Vec;

/// A contiguous growable array type with heap-allocated contents, guaranteed one element, written `NonEmptyVec<T>`,
/// short for 'non-empty vector'.
#[derive(Debug, PartialEq)]
pub struct NonEmptyVec<T> {
    head: T,
    tail: Vec<T>,
}

/// Alias for `NonEmptyVec<T>`.
///
/// A contiguous growable array type with heap-allocated contents, guaranteed one element, written `Nev<T>`,
/// short for 'non-empty vector'.
pub type Nev<T> = NonEmptyVec<T>;

impl<T> NonEmptyVec<T> {
    /// Constructs a new, non-empty `NonEmptyVec<T>`.
    ///
    /// # Arguments
    /// * `item`: The initial item for the `NonEmptyVec<T>`.
    ///
    /// # Examples
    /// ```
    /// use addon::vec::NonEmptyVec;
    /// let mut non_empty_vec: NonEmptyVec<i32> = NonEmptyVec::new(1);
    /// ```
    pub fn new(item: T) -> Self {
        Self {
            head: item,
            tail: Vec::new(),
        }
    }

    /// Appends an element to the back of a non-empty vector.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let mut nev = nev![1, 2];
    /// nev.push(3);
    /// assert_eq!(nev, nev![1, 2, 3]);
    /// ```
    ///
    /// # Time complexity
    /// Takes amortized *O*(1) time. If the vector's length would exceed its capacity after the push,
    /// *O*(capacity) time is taken to copy the vector’s elements to a larger allocation.
    /// This expensive operation is offset by the capacity *O*(1) insertions it allows.
    pub fn push(&mut self, value: T) {
        self.tail.push(value);
    }

    /// Appends an element to the front of a non-empty vector.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let mut nev = nev![2, 3];
    /// nev.push_front(1);
    /// assert_eq!(nev, nev![1, 2, 3]);
    /// ```
    ///
    /// # Time complexity
    /// Because this shifts over the remaining elements, it has a worst-case performance of *O*(n)
    pub fn push_front(&mut self, value: T) {
        self.tail
            .insert(0, core::mem::replace(&mut self.head, value));
    }

    /// Removes the last element from a non-empty vector and returns it, or `None` if there is only one element left.
    ///
    /// If you’d like to pop the first element, consider using `pop_front` instead.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let mut nev = nev![1, 2, 3];
    /// assert_eq!(nev.pop(), Some(3));
    /// assert_eq!(nev, nev![1, 2]);
    /// ```
    ///
    /// # Time complexity
    /// Takes *O*(1) time.
    pub fn pop(&mut self) -> Option<T> {
        self.tail.pop()
    }

    /// Removes the first element from a non-empty vector and returns it, or `None` if there is only one element left.
    ///
    /// If you'd like to pop the last element, consider using `pop` instead.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let mut nev = nev![1, 2, 3];
    /// assert_eq!(nev.pop_front(), Some(1));
    /// assert_eq!(nev, nev![2, 3]);
    /// ```
    ///
    /// # Time complexity
    /// Because this shifts over the remaining elements, it has a worst-case performance of *O*(n)
    pub fn pop_front(&mut self) -> Option<T> {
        if self.tail.is_empty() {
            return None;
        }
        Some(core::mem::replace(&mut self.head, self.tail.remove(0)))
    }

    /// Returns the number of elements in the non-empty vector, also referred to as its 'length'.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let a = nev![1, 2, 3];
    /// assert_eq!(a.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.tail.len() + 1
    }

    /// To satisfy clippy.
    ///
    /// Returns `false` because a `NonEmptyVec` cannot be empty.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns the first element of the `NonEmptyVec<T>`.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let v = nev![10, 40, 30];
    /// assert_eq!(&10, v.first());
    /// ```
    pub fn first(&self) -> &T {
        &self.head
    }

    /// Returns the last element of the `NonEmptyVec<T>`.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let v = nev![10, 40, 30];
    /// assert_eq!(&30, v.last());
    /// ```
    pub fn last(&self) -> &T {
        self.tail.last().unwrap_or(&self.head)
    }

    /// Returns an iterator over the `NonEmptyVec<T>`.
    ///
    /// The iterator yields all items from start to end.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let x = nev![1, 2, 4];
    /// let mut iterator = x.iter();
    ///
    /// assert_eq!(iterator.next(), Some(&1));
    /// assert_eq!(iterator.next(), Some(&2));
    /// assert_eq!(iterator.next(), Some(&4));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        core::iter::once(&self.head).chain(self.tail.iter())
    }

    /// Returns `true` if the slice contains an element with the given value.
    ///
    /// This operation is *O*(n).
    ///
    /// Note that if you have a sorted slice, `binary_search` may be faster.
    ///
    /// # Examples
    /// ```
    /// let n = [10, 40, 30];
    /// assert!(n.contains(&30));
    /// assert!(!n.contains(&50));
    /// ```
    ///
    /// If you do not have a `&T`, but some other value that you can compare with one (for example, `String` implements `PartialEq<str>`), you can use `iter().any`:
    /// ```
    /// let n = [String::from("hello"), String::from("world")]; // slice of `String`
    /// assert!(n.iter().any(|e| e == "hello")); // search with `&str`
    /// assert!(!n.iter().any(|e| e == "hi"));
    /// ```
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.tail.contains(x) || self.head == *x
    }

    /// Inserts an element at position index within the vector, shifting all elements after it to the right.
    ///
    /// # Panics
    /// Panics if `index > len`.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let mut nev = nev!['a', 'b', 'c'];
    /// nev.insert(1, 'd');
    /// assert_eq!(nev, ['a', 'd', 'b', 'c']);
    /// nev.insert(4, 'e');
    /// assert_eq!(nev, ['a', 'd', 'b', 'c', 'e']);
    /// ```
    ///
    /// # Time complexity
    /// Takes O([`NonEmptyVec::len`]) time. All items after the insertion index must be shifted to the right.
    /// In the worst case, all elements are shifted when the insertion index is 0. (see [`NonEmptyVec::push_front`] for more details)
    pub fn insert(&mut self, index: usize, item: T) {
        if index == 0 {
            return self.push_front(item);
        }
        self.tail.insert(index - 1, item);
    }
}

impl<T> Extend<T> for NonEmptyVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.tail.extend(iter);
    }
}

impl<T> core::ops::Index<usize> for NonEmptyVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            return &self.head;
        }
        &self.tail[index - 1]
    }
}

impl<T: PartialEq> PartialEq<[T]> for NonEmptyVec<T> {
    fn eq(&self, other: &[T]) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for i in 0..self.len() {
            if self[i] != other[i] {
                return false;
            }
        }
        true
    }
}

impl<T: PartialEq, const N: usize> PartialEq<[T; N]> for NonEmptyVec<T> {
    fn eq(&self, other: &[T; N]) -> bool {
        self == other.as_slice()
    }
}

impl<T> From<NonEmptyVec<T>> for Vec<T> {
    fn from(val: NonEmptyVec<T>) -> Self {
        let mut vec = alloc::vec![val.head];
        vec.extend(val.tail);
        vec
    }
}

impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;
    type IntoIter = core::iter::Chain<core::iter::Once<T>, alloc::vec::IntoIter<T>>;

    /// Creates a consuming iterator, that is, one that moves each value out of the non-empty vector (from start to end).
    /// The non-empty vector cannot be used after calling this.
    ///
    /// # Examples
    /// ```
    /// use addon::nev;
    /// let n = nev!["a".to_string(), "b".to_string()];
    /// let mut n_iter = n.into_iter();
    ///
    /// let first_element: Option<String> = n_iter.next();
    ///
    /// assert_eq!(first_element, Some("a".to_string()));
    /// assert_eq!(n_iter.next(), Some("b".to_string()));
    /// assert_eq!(n_iter.next(), None);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(self.head).chain(self.tail)
    }
}
