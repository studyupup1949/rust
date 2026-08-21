use number_traits::Num;
use vec2;

use super::{AABB2, copy, new_identity};


/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// let b = aabb2::new([-1, -1], [1, 1]);
/// assert_eq!(
///     aabb2::expand_point(&mut a, &b, &[2, 2]),
///     &AABB2{ min: [-1, -1], max: [2, 2] }
/// );
/// ~~~
#[inline]
pub fn expand_point<'out, 'a, 'b, T>(out: &'out mut AABB2<T>, aabb2: &'a AABB2<T>, p: &'b [T; 2]) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    vec2::min(&mut out.min, &aabb2.min, p);
    vec2::max(&mut out.max, &aabb2.max, p);
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// let b = aabb2::new([-1, -1], [1, 1]);
/// assert_eq!(
///     aabb2::expand_vector(&mut a, &b, &[1, 1]),
///     &AABB2{ min: [-2, -2], max: [2, 2] }
/// );
/// ~~~
#[inline]
pub fn expand_vector<'out, 'a, 'b, T>(out: &'out mut AABB2<T>, aabb2: &'a AABB2<T>, v: &'b [T; 2]) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    vec2::sub(&mut out.min, &aabb2.min, v);
    vec2::add(&mut out.max, &aabb2.max, v);
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// let b = aabb2::new([-1, -1], [1, 1]);
/// assert_eq!(
///     aabb2::expand_scalar(&mut a, &b, 1),
///     &AABB2{ min: [-2, -2], max: [2, 2] }
/// );
/// ~~~
#[inline]
pub fn expand_scalar<'out, 'a, T>(out: &'out mut AABB2<T>, aabb2: &'a AABB2<T>, s: T) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    vec2::ssub(&mut out.min, &aabb2.min, s);
    vec2::sadd(&mut out.max, &aabb2.max, s);
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// let b = aabb2::new([-1, -1], [0, 0]);
/// let c = aabb2::new([0, 0], [1, 1]);
/// assert_eq!(
///     aabb2::union(&mut a, &b, &c),
///     &AABB2{ min: [-1, -1], max: [1, 1] }
/// );
/// ~~~
#[inline]
pub fn union<'out, 'a, 'b, T>(out: &'out mut AABB2<T>, a: &'a AABB2<T>, b: &'b AABB2<T>) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    vec2::min(&mut out.min, &a.min, &b.min);
    vec2::max(&mut out.max, &a.max, &b.max);
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity::<u8>();
/// assert_eq!(
///     aabb2::clear(&mut a),
///     &AABB2{ min: [255, 255], max: [0, 0] }
/// );
/// ~~~
#[inline]
pub fn clear<'out, T>(out: &'out mut AABB2<T>) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    let min = T::min_value();
    let max = T::max_value();
    vec2::set(&mut out.min, max, max);
    vec2::set(&mut out.max, min, min);
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new([-1, -1], [1, 1]);
/// assert!(aabb2::contains(&a, &[0, 0]));
/// assert!(aabb2::contains(&a, &[0, 1]));
/// assert!(aabb2::contains(&a, &[-1, 1]));
/// assert!(!aabb2::contains(&a, &[-2, 1]));
/// assert!(!aabb2::contains(&a, &[0, 2]));
/// ~~~
#[inline]
pub fn contains<'a, 'b, T>(aabb: &'a AABB2<T>, p: &'b [T; 2]) -> bool
    where T: Copy + Num,
{
    return !(
        &p[0] < &aabb.min[0] || &p[0] > &aabb.max[0] ||
        &p[1] < &aabb.min[1] || &p[1] > &aabb.max[1]
    );
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new([1, 1], [2, 2]);
/// let mut b = aabb2::new([-2, -2], [-1, -1]);
/// let mut c = aabb2::new([-1, -1], [1, 1]);

/// assert!(!aabb2::intersects(&a, &b));
/// assert!(aabb2::intersects(&a, &c));
/// assert!(aabb2::intersects(&b, &c));
/// ~~~
#[inline]
pub fn intersects<'a, 'b, T>(a: &'a AABB2<T>, b: &'b AABB2<T>) -> bool
    where T: Copy + Num,
{
    return !(
        &b.max[0] < &a.min[0] || &b.min[0] > &a.max[0] ||
        &b.max[1] < &a.min[1] || &b.min[1] > &a.max[1]
    );
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// assert_eq!(
///     aabb2::from_points(&mut a, &[[-1, 1], [1, 1], [1, -1], [-1, -1]]),
///     &aabb2::new([-1, -1], [1, 1])
/// );
/// ~~~
#[inline]
pub fn from_points<'out, 'a, T>(out: &'out mut AABB2<T>, points: &'a [[T; 2]]) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    let mut tmp = new_identity();

    for p in points {
        expand_point(out, &tmp, p);
        copy(&mut tmp, out);
    }

    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// assert_eq!(
///     aabb2::from_center_size(&mut a, &[0, 0], &[2, 2]),
///     &aabb2::new([-1, -1], [1, 1])
/// );
/// ~~~
#[inline]
pub fn from_center_size<'out, 'a, 'b, T>(out: &'out mut AABB2<T>, center: &'a [T; 2], size: &'b [T; 2]) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    let hx = size[0] / T::from_usize(2);
    let hy = size[1] / T::from_usize(2);

    out.min[0] = center[0] - hx;
    out.min[1] = center[1] - hy;
    out.max[0] = center[0] + hx;
    out.max[1] = center[1] + hy;
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// assert_eq!(
///     aabb2::from_center_radius(&mut a, &[0, 0], 1),
///     &aabb2::new([-1, -1], [1, 1])
/// );
/// ~~~
#[inline]
pub fn from_center_radius<'out, 'a, T>(out: &'out mut AABB2<T>, center: &'a [T; 2], radius: T) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    out.min[0] = center[0] - radius;
    out.min[1] = center[1] - radius;
    out.max[0] = center[0] + radius;
    out.max[1] = center[1] + radius;
    out
}
