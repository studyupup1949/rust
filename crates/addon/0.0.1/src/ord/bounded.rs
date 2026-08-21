/// A value guaranteed to be within a `[min, max]` range.
#[derive(Debug, PartialEq)]
pub struct Bounded<T>
where
    T: PartialOrd,
{
    value: T,
    min: T,
    max: T,
}

impl<T> PartialOrd for Bounded<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T> Bounded<T>
where
    T: PartialOrd,
{
    fn panic_if_out_of_bounds(value: &T, min: &T, max: &T) {
        assert!(!(value < min || value > max), "value out of [min, max] bounds");
    }

    /// Constructs a new `Bounded<T>`
    ///
    /// # Arguments
    /// * `value`: The value of the type
    /// * `min`: The minimal bound
    /// * `max`: The maximal bound
    ///
    /// # Panics
    /// Panics when `value` is out of bounds (set by `min` and `max`)
    ///
    /// # Examples
    /// ```
    /// use addon::ord::Bounded;
    /// let bounded: Bounded<i32> = Bounded::new(5, 1, 10);
    /// ```
    pub fn new(value: T, min: T, max: T) -> Self {
        Self::panic_if_out_of_bounds(&value, &min, &max);
        Self { value, min, max }
    }

    /// Returns the value of a `Bounded<T>`, borrowed.
    ///
    /// # Examples
    /// ```
    /// use addon::ord::Bounded;
    /// let bounded: Bounded<i32> = Bounded::new(5, 1, 10);
    /// assert_eq!(bounded.value(), &5);
    /// ```
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the minimal bound of a `Bounded<T>`, borrowed.
    ///
    /// # Examples
    /// ```
    /// use addon::ord::Bounded;
    /// let bounded: Bounded<i32> = Bounded::new(5, 1, 10);
    /// assert_eq!(bounded.min(), &1)
    /// ```
    pub fn min(&self) -> &T {
        &self.min
    }

    /// Returns the maximal bound of a `Bounded<T>`, borrowed.
    ///
    /// # Examples
    /// ```
    /// use addon::ord::Bounded;
    /// let bounded: Bounded<i32> = Bounded::new(5, 1, 10);
    /// assert_eq!(bounded.max(), &10)
    /// ```
    pub fn max(&self) -> &T {
        &self.max
    }

    /// Sets the value of a `Bounded<T>`
    ///
    /// # Panics
    /// Panics when `value` is out of bounds (set by `min` and `max` at construction)
    ///
    /// # Examples
    /// ```
    /// use addon::ord::Bounded;
    /// let mut bounded: Bounded<i32> = Bounded::new(5, 1, 10);
    /// bounded.set_value(9);
    /// assert_eq!(bounded.value(), &9);
    /// ```
    pub fn set_value(&mut self, value: T) {
        Self::panic_if_out_of_bounds(&value, &self.min, &self.max);
        self.value = value;
    }
}
