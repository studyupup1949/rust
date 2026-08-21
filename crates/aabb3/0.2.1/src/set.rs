use core::ops::{Add, Div, Sub};

use num_traits::{Bounded, FromPrimitive};
use vec3;

use super::{expand_point, new_identity, AABB3};

///
/// # Examples
/// ```
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// assert_eq!(aabb3::set(&mut a, &[0.0, 0.0, 0.0], &[1.0, 1.0, 1.0]), &AABB3{ min: [0.0, 0.0, 0.0], max: [1.0, 1.0, 1.0] });
/// ```
#[inline]
pub fn set<'out, T>(out: &'out mut AABB3<T>, min: &[T; 3], max: &[T; 3]) -> &'out mut AABB3<T>
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
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity::<u8>();
/// assert_eq!(
///     aabb3::set_identity(&mut a),
///     &AABB3{ min: [255, 255, 255], max: [0, 0, 0] }
/// );
/// ```
#[inline]
pub fn set_identity<'out, T>(out: &'out mut AABB3<T>) -> &'out mut AABB3<T>
where
    T: Clone + Bounded,
{
    let min = T::min_value();
    let max = T::max_value();
    vec3::set(&mut out.min, max.clone(), max.clone(), max);
    vec3::set(&mut out.max, min.clone(), min.clone(), min);
    out
}
///
/// # Examples
/// ```
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// assert_eq!(
///     aabb3::set_points(&mut a, &[[-1.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, -1.0, 0.0]]),
///     &aabb3::new([-1.0, -1.0, 0.0], [1.0, 1.0, 0.0])
/// );
/// ```
#[inline]
pub fn set_points<'out, T>(out: &'out mut AABB3<T>, points: &[[T; 3]]) -> &'out mut AABB3<T>
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
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// assert_eq!(
///     aabb3::set_center_size(&mut a, &[0.0, 0.0, 0.0], &[2.0, 2.0, 2.0]),
///     &aabb3::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0])
/// );
/// ```
#[inline]
pub fn set_center_size<'out, T>(
    out: &'out mut AABB3<T>,
    center: &[T; 3],
    size: &[T; 3],
) -> &'out mut AABB3<T>
where
    T: FromPrimitive,
    for<'a, 'b> &'a T: Div<&'b T, Output = T> + Add<&'b T, Output = T> + Sub<&'b T, Output = T>,
{
    let hx = &size[0] / &T::from_usize(2).unwrap();
    let hy = &size[1] / &T::from_usize(2).unwrap();

    out.min[0] = &center[0] - &hx;
    out.min[1] = &center[1] - &hy;
    out.min[2] = &center[2] - &hy;
    out.max[0] = &center[0] + &hx;
    out.max[1] = &center[1] + &hy;
    out.max[2] = &center[2] + &hy;
    out
}
///
/// # Examples
/// ```
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// assert_eq!(
///     aabb3::set_center_radius(&mut a, &[0.0, 0.0, 0.0], &1.0),
///     &aabb3::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0])
/// );
/// ```
#[inline]
pub fn set_center_radius<'out, T>(
    out: &'out mut AABB3<T>,
    center: &[T; 3],
    radius: &T,
) -> &'out mut AABB3<T>
where
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T>,
{
    out.min[0] = &center[0] - radius;
    out.min[1] = &center[1] - radius;
    out.min[2] = &center[2] - radius;
    out.max[0] = &center[0] + radius;
    out.max[1] = &center[1] + radius;
    out.max[2] = &center[2] + radius;
    out
}
