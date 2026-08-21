use num_traits::Bounded;

use super::AABB3;

///
/// # Examples
/// ```
/// use aabb3::{self, AABB3};
/// assert_eq!(&aabb3::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), &AABB3{ min: [0.0, 0.0, 0.0], max: [1.0, 1.0, 1.0] });
/// ```
#[inline(always)]
pub fn new<T>(min: [T; 3], max: [T; 3]) -> AABB3<T> {
    AABB3 { min: min, max: max }
}
///
/// # Examples
/// ```
/// use aabb3::{self, AABB3};
/// assert_eq!(&aabb3::new_identity::<u8>(), &AABB3{ min: [255, 255, 255], max: [0, 0, 0] });
/// ```
#[inline(always)]
pub fn new_identity<T>() -> AABB3<T>
where
    T: Clone + Bounded,
{
    let min = T::min_value();
    let max = T::max_value();

    AABB3 {
        min: [max.clone(), max.clone(), max],
        max: [min.clone(), min.clone(), min],
    }
}
