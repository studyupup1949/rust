use number_traits::Num;
use vec3;

use super::{copy, new_identity, AABB3};

/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// let b = aabb3::new([-1, -1, -1], [1, 1, 1]);
/// assert_eq!(
///     aabb3::expand_point(&mut a, &b, &[2, 2, 2]),
///     &AABB3{ min: [-1, -1, -1], max: [2, 2, 2] }
/// );
/// ~~~
#[inline]
pub fn expand_point<'out, 'a, 'b, T>(
    out: &'out mut AABB3<T>,
    aabb3: &'a AABB3<T>,
    p: &'b [T; 3],
) -> &'out mut AABB3<T>
where
    T: Copy + Num,
{
    vec3::min(&mut out.min, &aabb3.min, p);
    vec3::max(&mut out.max, &aabb3.max, p);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// let b = aabb3::new([-1, -1, -1], [1, 1, 1]);
/// assert_eq!(
///     aabb3::expand_vector(&mut a, &b, &[1, 1, 1]),
///     &AABB3{ min: [-2, -2, -2], max: [2, 2, 2] }
/// );
/// ~~~
#[inline]
pub fn expand_vector<'out, 'a, 'b, T>(
    out: &'out mut AABB3<T>,
    aabb3: &'a AABB3<T>,
    v: &'b [T; 3],
) -> &'out mut AABB3<T>
where
    T: Copy + Num,
{
    vec3::sub(&mut out.min, &aabb3.min, v);
    vec3::add(&mut out.max, &aabb3.max, v);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// let b = aabb3::new([-1, -1, -1], [1, 1, 1]);
/// assert_eq!(
///     aabb3::expand_scalar(&mut a, &b, 1),
///     &AABB3{ min: [-2, -2, -2], max: [2, 2, 2] }
/// );
/// ~~~
#[inline]
pub fn expand_scalar<'out, 'a, T>(
    out: &'out mut AABB3<T>,
    aabb3: &'a AABB3<T>,
    s: T,
) -> &'out mut AABB3<T>
where
    T: Copy + Num,
{
    vec3::ssub(&mut out.min, &aabb3.min, s);
    vec3::sadd(&mut out.max, &aabb3.max, s);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// let b = aabb3::new([-1, -1, -1], [0, 0, 0]);
/// let c = aabb3::new([0, 0, 0], [1, 1, 1]);
/// assert_eq!(
///     aabb3::union(&mut a, &b, &c),
///     &AABB3{ min: [-1, -1, -1], max: [1, 1, 1] }
/// );
/// ~~~
#[inline]
pub fn union<'out, 'a, 'b, T>(
    out: &'out mut AABB3<T>,
    a: &'a AABB3<T>,
    b: &'b AABB3<T>,
) -> &'out mut AABB3<T>
where
    T: Copy + Num,
{
    vec3::min(&mut out.min, &a.min, &b.min);
    vec3::max(&mut out.max, &a.max, &b.max);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity::<u8>();
/// assert_eq!(
///     aabb3::clear(&mut a),
///     &AABB3{ min: [255, 255, 255], max: [0, 0, 0] }
/// );
/// ~~~
#[inline]
pub fn clear<'out, T>(out: &'out mut AABB3<T>) -> &'out mut AABB3<T>
where
    T: Copy + Num,
{
    let min = T::min_value();
    let max = T::max_value();
    vec3::set(&mut out.min, max, max, max);
    vec3::set(&mut out.max, min, min, min);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new([-1, -1, -1], [1, 1, 1]);
/// assert!(aabb3::contains_point(&a, &[0, 0, 0]));
/// assert!(aabb3::contains_point(&a, &[0, 1, 0]));
/// assert!(aabb3::contains_point(&a, &[-1, 1, 0]));
/// assert!(!aabb3::contains_point(&a, &[-2, 1, 0]));
/// assert!(!aabb3::contains_point(&a, &[0, 2, 0]));
/// ~~~
#[inline]
pub fn contains_point<'a, 'b, T>(aabb: &'a AABB3<T>, p: &'b [T; 3]) -> bool
where
    T: Copy + Num,
{
    return !(&p[0] < &aabb.min[0] || &p[0] > &aabb.max[0] || &p[1] < &aabb.min[1]
        || &p[1] > &aabb.max[1]);
}
/// # Examples
/// ~~~
/// use aabb3;
/// let a = aabb3::new([-2, -2, -2], [2, 2, 2]);
/// let b = aabb3::new([-1, -1, -1], [1, 1, 1]);
/// let c = aabb3::new([0, 0, 0], [3, 3, 3]);
/// assert!(aabb3::contains(&a, &b));
/// assert!(!aabb3::contains(&a, &c));
/// ~~~
#[inline]
pub fn contains<'a, 'b, T>(a: &'a AABB3<T>, b: &'b AABB3<T>) -> bool
where
    T: Copy + Num,
{
    return !(&a.min[0] >= &b.min[0] || &a.max[0] <= &b.max[0] || &a.min[1] >= &b.min[1]
        || &a.max[1] <= &b.max[1]);
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new([1, 1, 1], [2, 2, 2]);
/// let mut b = aabb3::new([-2, -2, -2], [-1, -1, -1]);
/// let mut c = aabb3::new([-1, -1, -1], [1, 1, 1]);

/// assert!(!aabb3::intersects(&a, &b));
/// assert!(aabb3::intersects(&a, &c));
/// assert!(aabb3::intersects(&b, &c));
/// ~~~
#[inline]
pub fn intersects<'a, 'b, T>(a: &'a AABB3<T>, b: &'b AABB3<T>) -> bool
where
    T: Copy + Num,
{
    return !(&b.max[0] < &a.min[0] || &b.min[0] > &a.max[0] || &b.max[1] < &a.min[1]
        || &b.min[1] > &a.max[1]);
}
/// # Examples
/// ~~~
/// use aabb3;
/// let a = aabb3::new([0, 0, 0], [2, 2, 2]);
/// let mut v = [0, 0, 0];
/// assert_eq!(aabb3::center(&mut v, &a), &[1, 1, 1]);
/// ~~~
#[inline]
pub fn center<'out, 'a, T>(out: &'out mut [T; 3], aabb: &'a AABB3<T>) -> &'out mut [T; 3]
where
    T: Copy + Num,
{
    out[0] = (aabb.min[0] + aabb.max[0]) / T::from_usize(2);
    out[1] = (aabb.min[1] + aabb.max[1]) / T::from_usize(2);
    out[2] = (aabb.min[2] + aabb.max[2]) / T::from_usize(2);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// assert_eq!(
///     aabb3::from_points(&mut a, &[[-1, 1, 0], [1, 1, 0], [1, -1, 0], [-1, -1, 0]]),
///     &aabb3::new([-1, -1, 0], [1, 1, 0])
/// );
/// ~~~
#[inline]
pub fn from_points<'out, 'a, T>(out: &'out mut AABB3<T>, points: &'a [[T; 3]]) -> &'out mut AABB3<T>
where
    T: Copy + Num,
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
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// assert_eq!(
///     aabb3::from_center_size(&mut a, &[0, 0, 0], &[2, 2, 2]),
///     &aabb3::new([-1, -1, -1], [1, 1, 1])
/// );
/// ~~~
#[inline]
pub fn from_center_size<'out, 'a, 'b, T>(
    out: &'out mut AABB3<T>,
    center: &'a [T; 3],
    size: &'b [T; 3],
) -> &'out mut AABB3<T>
where
    T: Copy + Num,
{
    let hx = size[0] / T::from_usize(2);
    let hy = size[1] / T::from_usize(2);
    let hz = size[2] / T::from_usize(2);

    out.min[0] = center[0] - hx;
    out.min[1] = center[1] - hy;
    out.min[2] = center[2] - hz;
    out.max[0] = center[0] + hx;
    out.max[1] = center[1] + hy;
    out.max[2] = center[2] + hz;
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// assert_eq!(
///     aabb3::from_center_radius(&mut a, &[0, 0, 0], 1),
///     &aabb3::new([-1, -1, -1], [1, 1, 1])
/// );
/// ~~~
#[inline]
pub fn from_center_radius<'out, 'a, T>(
    out: &'out mut AABB3<T>,
    center: &'a [T; 3],
    radius: T,
) -> &'out mut AABB3<T>
where
    T: Copy + Num,
{
    out.min[0] = center[0] - radius;
    out.min[1] = center[1] - radius;
    out.min[2] = center[2] - radius;
    out.max[0] = center[0] + radius;
    out.max[1] = center[1] + radius;
    out.max[2] = center[2] + radius;
    out
}
