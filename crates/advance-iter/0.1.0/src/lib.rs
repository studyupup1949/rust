//! This crate provides two structs, [`Advance`] and [`CountingAdvance`], to help with consuming iterators one step at
//! a time. Refer to their respective documentation for more information.

/// Wrapper around an iterator. Has to be advanced using the
/// [`advance`] method, which will cache the iterator's next element
/// in `self.current`.
///
/// See also [`CountingAdvance`], a similar adapter that keeps track of how
/// many times it has been advanced.
///
/// [`advance`]: Advance::advance
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Advance<I: Iterator> {
    current: Option<I::Item>,
    iter: I,
}

impl<I: Iterator> Advance<I> {
    /// Wraps the given iterator in an [`Advance`] adapter. This initiates
    /// `self.current` with the iterator's first element (if any).
    #[inline]
    pub fn new(mut iter: I) -> Self {
        Self {
            current: iter.next(),
            iter,
        }
    }

    #[inline]
    pub fn advance(&mut self) {
        self.current = self.iter.next();
    }

    #[inline]
    pub fn current(&self) -> Option<&I::Item> {
        self.current.as_ref()
    }

    #[inline]
    pub fn current_mut(&mut self) -> Option<&mut I::Item> {
        self.current.as_mut()
    }
}

/// Wrapper around an iterator. Has to be advanced using the
/// [`advance`][adv_fn] method, which will cache the iterator's next element
/// in `self.current` and increment `self.counter`.
///
/// See also [`Advance`], a similar adapter that does not keep track of how
/// many times it has been advanced.
///
/// [adv_fn]: CountingAdvance::advance
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CountingAdvance<I: Iterator> {
    counter: usize,
    current: Option<I::Item>,
    iter: I,
}

impl<I: Iterator> CountingAdvance<I> {
    /// Wraps the given iterator in a [`CountingAdvance`] adapter. This
    /// initiates `self.current` with the iterator's first element (if any)
    /// and starts the counter at zero.
    #[inline]
    pub fn new(mut iter: I) -> Self {
        Self {
            counter: 0,
            current: iter.next(),
            iter,
        }
    }

    #[inline]
    pub fn advance(&mut self) {
        self.counter += 1;
        self.current = self.iter.next();
    }

    #[inline]
    pub fn counter(&self) -> usize {
        self.counter
    }

    #[inline]
    pub fn current(&self) -> Option<&I::Item> {
        self.current.as_ref()
    }

    #[inline]
    pub fn current_mut(&mut self) -> Option<&mut I::Item> {
        self.current.as_mut()
    }
}
