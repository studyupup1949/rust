use num_traits::Bounded;

use super::AABB2;

///
/// # Examples
/// ```
/// use aabb2::{self, AABB2};
/// assert_eq!(&aabb2::new([0.0, 0.0], [1.0, 1.0]), &AABB2{ min: [0.0, 0.0], max: [1.0, 1.0] });
/// ```
#[inline(always)]
pub fn new<T>(min: [T; 2], max: [T; 2]) -> AABB2<T> {
    AABB2 { min: min, max: max }
}
///
/// # Examples
/// ```
/// use aabb2::{self, AABB2};
/// assert_eq!(&aabb2::new_identity::<u8>(), &AABB2{ min: [255, 255], max: [0, 0] });
/// ```
#[inline(always)]
pub fn new_identity<T>() -> AABB2<T>
where
    T: Clone + Bounded,
{
    let min = T::min_value();
    let max = T::max_value();

    AABB2 {
        min: [max.clone(), max],
        max: [min.clone(), min],
    }
}
