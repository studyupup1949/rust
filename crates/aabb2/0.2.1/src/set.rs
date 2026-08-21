use core::ops::{Add, Div, Sub};

use num_traits::{Bounded, FromPrimitive};
use vec2;

use super::{expand_point, new_identity, AABB2};

///
/// # Examples
/// ```
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// assert_eq!(aabb2::set(&mut a, &[0.0, 0.0], &[1.0, 1.0]), &AABB2{ min: [0.0, 0.0], max: [1.0, 1.0] });
/// ```
#[inline]
pub fn set<'out, T>(out: &'out mut AABB2<T>, min: &[T; 2], max: &[T; 2]) -> &'out mut AABB2<T>
where
    T: Clone,
{
    out.min.clone_from(min);
    out.max.clone_from(max);
    out
}
///
/// # Examples
/// ```
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity::<u8>();
/// assert_eq!(
///     aabb2::set_identity(&mut a),
///     &AABB2{ min: [255, 255], max: [0, 0] }
/// );
/// ```
#[inline]
pub fn set_identity<'out, T>(out: &'out mut AABB2<T>) -> &'out mut AABB2<T>
where
    T: Clone + Bounded,
{
    let min = T::min_value();
    let max = T::max_value();
    vec2::set(&mut out.min, max.clone(), max);
    vec2::set(&mut out.max, min.clone(), min);
    out
}
///
/// # Examples
/// ```
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// assert_eq!(
///     aabb2::set_points(&mut a, &[[-1.0, 1.0], [1.0, 1.0], [1.0, -1.0], [-1.0, -1.0]]),
///     &aabb2::new([-1.0, -1.0], [1.0, 1.0])
/// );
/// ```
#[inline]
pub fn set_points<'out, T>(out: &'out mut AABB2<T>, points: &[[T; 2]]) -> &'out mut AABB2<T>
where
    T: Clone + Bounded + PartialOrd,
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T>,
{
    let mut tmp = new_identity();

    for p in points {
        expand_point(out, &tmp, p);
        tmp.clone_from(out);
    }

    out
}
///
/// # Examples
/// ```
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// assert_eq!(
///     aabb2::set_center_size(&mut a, &[0.0, 0.0], &[2.0, 2.0]),
///     &aabb2::new([-1.0, -1.0], [1.0, 1.0])
/// );
/// ```
#[inline]
pub fn set_center_size<'out, T>(
    out: &'out mut AABB2<T>,
    center: &[T; 2],
    size: &[T; 2],
) -> &'out mut AABB2<T>
where
    T: FromPrimitive,
    for<'a, 'b> &'a T: Div<&'b T, Output = T> + Add<&'b T, Output = T> + Sub<&'b T, Output = T>,
{
    let hx = &size[0] / &T::from_usize(2).unwrap();
    let hy = &size[1] / &T::from_usize(2).unwrap();

    out.min[0] = &center[0] - &hx;
    out.min[1] = &center[1] - &hy;
    out.max[0] = &center[0] + &hx;
    out.max[1] = &center[1] + &hy;
    out
}
///
/// # Examples
/// ```
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// assert_eq!(
///     aabb2::set_center_radius(&mut a, &[0.0, 0.0], &1.0),
///     &aabb2::new([-1.0, -1.0], [1.0, 1.0])
/// );
/// ```
#[inline]
pub fn set_center_radius<'out, T>(
    out: &'out mut AABB2<T>,
    center: &[T; 2],
    radius: &T,
) -> &'out mut AABB2<T>
where
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T>,
{
    out.min[0] = &center[0] - radius;
    out.min[1] = &center[1] - radius;
    out.max[0] = &center[0] + radius;
    out.max[1] = &center[1] + radius;
    out
}
