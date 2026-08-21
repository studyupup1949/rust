use core::ops::{Add, Div, Sub};

use num_traits::{Bounded, FromPrimitive};
use vec3;

use super::{set_identity, AABB3};

/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// let b = aabb3::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
/// assert_eq!(
///     aabb3::expand_point(&mut a, &b, &[2.0, 2.0, 2.0]),
///     &AABB3{ min: [-1.0, -1.0, -1.0], max: [2.0, 2.0, 2.0] }
/// );
/// ~~~
#[inline]
pub fn expand_point<'out, T>(
    out: &'out mut AABB3<T>,
    aabb3: &AABB3<T>,
    p: &[T; 3],
) -> &'out mut AABB3<T>
where
    T: Clone + PartialOrd,
{
    vec3::min(&mut out.min, &aabb3.min, p);
    vec3::max(&mut out.max, &aabb3.max, p);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// let b = aabb3::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
/// assert_eq!(
///     aabb3::expand_vector(&mut a, &b, &[1.0, 1.0, 1.0]),
///     &AABB3{ min: [-2.0, -2.0, -2.0], max: [2.0, 2.0, 2.0] }
/// );
/// ~~~
#[inline]
pub fn expand_vector<'out, T>(
    out: &'out mut AABB3<T>,
    aabb3: &AABB3<T>,
    v: &[T; 3],
) -> &'out mut AABB3<T>
where
    T: PartialOrd,
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T>,
{
    vec3::sub(&mut out.min, &aabb3.min, v);
    vec3::add(&mut out.max, &aabb3.max, v);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// let b = aabb3::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
/// assert_eq!(
///     aabb3::expand_scalar(&mut a, &b, &1.0),
///     &AABB3{ min: [-2.0, -2.0, -2.0], max: [2.0, 2.0, 2.0] }
/// );
/// ~~~
#[inline]
pub fn expand_scalar<'out, T>(
    out: &'out mut AABB3<T>,
    aabb3: &AABB3<T>,
    s: &T,
) -> &'out mut AABB3<T>
where
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T>,
{
    vec3::ssub(&mut out.min, &aabb3.min, s);
    vec3::sadd(&mut out.max, &aabb3.max, s);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// let b = aabb3::new([-1.0, -1.0, -1.0], [0.0, 0.0, 0.0]);
/// let c = aabb3::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
/// assert_eq!(
///     aabb3::union(&mut a, &b, &c),
///     &AABB3{ min: [-1.0, -1.0, -1.0], max: [1.0, 1.0, 1.0] }
/// );
/// ~~~
#[inline]
pub fn union<'out, T>(out: &'out mut AABB3<T>, a: &AABB3<T>, b: &AABB3<T>) -> &'out mut AABB3<T>
where
    T: Clone + PartialOrd,
{
    vec3::min(&mut out.min, &a.min, &b.min);
    vec3::max(&mut out.max, &a.max, &b.max);
    out
}
#[inline]
pub fn clear<'out, T>(out: &'out mut AABB3<T>) -> &'out mut AABB3<T>
where
    T: Clone + Bounded,
{
    set_identity(out)
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
/// assert!(aabb3::contains_point(&a, &[0.0, 0.0, 0.0]));
/// assert!(aabb3::contains_point(&a, &[0.0, 1.0, 0.0]));
/// assert!(aabb3::contains_point(&a, &[-1.0, 1.0, 0.0]));
/// assert!(!aabb3::contains_point(&a, &[-2.0, 1.0, 0.0]));
/// assert!(!aabb3::contains_point(&a, &[0.0, 2.0, 0.0]));
/// ~~~
#[inline]
pub fn contains_point<T>(aabb: &AABB3<T>, p: &[T; 3]) -> bool
where
    T: PartialOrd,
{
    return &p[0] >= &aabb.min[0]
        && &p[0] <= &aabb.max[0]
        && &p[1] >= &aabb.min[1]
        && &p[1] <= &aabb.max[1]
        && &p[2] >= &aabb.min[2]
        && &p[2] <= &aabb.max[2];
}
/// # Examples
/// ~~~
/// use aabb3;
/// let a = aabb3::new([-2.0, -2.0, -2.0], [2.0, 2.0, 2.0]);
/// let b = aabb3::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
/// let c = aabb3::new([0.0, 0.0, 0.0], [3.0, 3.0, 3.0]);
/// assert!(aabb3::contains(&a, &b));
/// assert!(!aabb3::contains(&a, &c));
/// ~~~
#[inline]
pub fn contains<T>(a: &AABB3<T>, b: &AABB3<T>) -> bool
where
    T: PartialOrd,
{
    return &a.min[0] < &b.min[0]
        && &a.max[0] > &b.max[0]
        && &a.min[1] < &b.min[1]
        && &a.max[1] > &b.max[1]
        && &a.min[2] < &b.min[2]
        && &a.max[2] > &b.max[2];
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);
/// let mut b = aabb3::new([-2.0, -2.0, -2.0], [-1.0, -1.0, -1.0]);
/// let mut c = aabb3::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

/// assert!(!aabb3::intersects(&a, &b));
/// assert!(aabb3::intersects(&a, &c));
/// assert!(aabb3::intersects(&b, &c));
/// ~~~
#[inline]
pub fn intersects<T>(a: &AABB3<T>, b: &AABB3<T>) -> bool
where
    T: PartialOrd,
{
    return &b.max[0] >= &a.min[0]
        && &b.min[0] <= &a.max[0]
        && &b.max[1] >= &a.min[1]
        && &b.min[1] <= &a.max[1]
        && &b.max[2] >= &a.min[2]
        && &b.min[2] <= &a.max[2];
}
/// # Examples
/// ~~~
/// use aabb3;
/// let a = aabb3::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
/// let mut v = [0.0, 0.0, 0.0];
/// assert_eq!(aabb3::center(&mut v, &a), &[1.0, 1.0, 1.0]);
/// ~~~
#[inline]
pub fn center<'out, T>(out: &'out mut [T; 3], aabb: &AABB3<T>) -> &'out mut [T; 3]
where
    T: FromPrimitive,
    for<'a, 'b> &'a T: Div<&'b T, Output = T> + Add<&'b T, Output = T> + Div<&'b T, Output = T>,
{
    out[0] = &(&aabb.min[0] + &aabb.max[0]) / &T::from_usize(2).unwrap();
    out[1] = &(&aabb.min[1] + &aabb.max[1]) / &T::from_usize(2).unwrap();
    out[2] = &(&aabb.min[2] + &aabb.max[2]) / &T::from_usize(2).unwrap();
    out
}
