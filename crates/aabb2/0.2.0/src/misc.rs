use core::ops::{Add, Div, Sub};

use num_traits::{Bounded, FromPrimitive};
use vec2;

use super::{set_identity, AABB2};

/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// let b = aabb2::new([-1.0, -1.0], [1.0, 1.0]);
/// assert_eq!(
///     aabb2::expand_point(&mut a, &b, &[2.0, 2.0]),
///     &AABB2{ min: [-1.0, -1.0], max: [2.0, 2.0] }
/// );
/// ~~~
#[inline]
pub fn expand_point<'out, T>(
    out: &'out mut AABB2<T>,
    aabb2: &AABB2<T>,
    p: &[T; 2],
) -> &'out mut AABB2<T>
where
    T: Clone + PartialOrd,
{
    vec2::min(&mut out.min, &aabb2.min, p);
    vec2::max(&mut out.max, &aabb2.max, p);
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// let b = aabb2::new([-1.0, -1.0], [1.0, 1.0]);
/// assert_eq!(
///     aabb2::expand_vector(&mut a, &b, &[1.0, 1.0]),
///     &AABB2{ min: [-2.0, -2.0], max: [2.0, 2.0] }
/// );
/// ~~~
#[inline]
pub fn expand_vector<'out, T>(
    out: &'out mut AABB2<T>,
    aabb2: &AABB2<T>,
    v: &[T; 2],
) -> &'out mut AABB2<T>
where
    T: PartialOrd,
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T>,
{
    vec2::sub(&mut out.min, &aabb2.min, v);
    vec2::add(&mut out.max, &aabb2.max, v);
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// let b = aabb2::new([-1.0, -1.0], [1.0, 1.0]);
/// assert_eq!(
///     aabb2::expand_scalar(&mut a, &b, &1.0),
///     &AABB2{ min: [-2.0, -2.0], max: [2.0, 2.0] }
/// );
/// ~~~
#[inline]
pub fn expand_scalar<'out, T>(
    out: &'out mut AABB2<T>,
    aabb2: &AABB2<T>,
    s: &T,
) -> &'out mut AABB2<T>
where
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T>,
{
    vec2::ssub(&mut out.min, &aabb2.min, s);
    vec2::sadd(&mut out.max, &aabb2.max, s);
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// let b = aabb2::new([-1.0, -1.0], [0.0, 0.0]);
/// let c = aabb2::new([0.0, 0.0], [1.0, 1.0]);
/// assert_eq!(
///     aabb2::union(&mut a, &b, &c),
///     &AABB2{ min: [-1.0, -1.0], max: [1.0, 1.0] }
/// );
/// ~~~
#[inline]
pub fn union<'out, T>(out: &'out mut AABB2<T>, a: &AABB2<T>, b: &AABB2<T>) -> &'out mut AABB2<T>
where
    T: Clone + PartialOrd,
{
    vec2::min(&mut out.min, &a.min, &b.min);
    vec2::max(&mut out.max, &a.max, &b.max);
    out
}
#[inline]
pub fn clear<'out, T>(out: &'out mut AABB2<T>) -> &'out mut AABB2<T>
where
    T: Clone + Bounded,
{
    set_identity(out)
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new([-1.0, -1.0], [1.0, 1.0]);
/// assert!(aabb2::contains_point(&a, &[0.0, 0.0]));
/// assert!(aabb2::contains_point(&a, &[0.0, 1.0]));
/// assert!(aabb2::contains_point(&a, &[-1.0, 1.0]));
/// assert!(!aabb2::contains_point(&a, &[-2.0, 1.0]));
/// assert!(!aabb2::contains_point(&a, &[0.0, 2.0]));
/// ~~~
#[inline]
pub fn contains_point<T>(aabb: &AABB2<T>, p: &[T; 2]) -> bool
where
    T: PartialOrd,
{
    return &p[0] >= &aabb.min[0]
        && &p[0] <= &aabb.max[0]
        && &p[1] >= &aabb.min[1]
        && &p[1] <= &aabb.max[1];
}
/// # Examples
/// ~~~
/// use aabb2;
/// let a = aabb2::new([-2.0, -2.0], [2.0, 2.0]);
/// let b = aabb2::new([-1.0, -1.0], [1.0, 1.0]);
/// let c = aabb2::new([0.0, 0.0], [3.0, 3.0]);
/// assert!(aabb2::contains(&a, &b));
/// assert!(!aabb2::contains(&a, &c));
/// ~~~
#[inline]
pub fn contains<T>(a: &AABB2<T>, b: &AABB2<T>) -> bool
where
    T: PartialOrd,
{
    return &a.min[0] < &b.min[0]
        && &a.max[0] > &b.max[0]
        && &a.min[1] < &b.min[1]
        && &a.max[1] > &b.max[1];
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new([1.0, 1.0], [2.0, 2.0]);
/// let mut b = aabb2::new([-2.0, -2.0], [-1.0, -1.0]);
/// let mut c = aabb2::new([-1.0, -1.0], [1.0, 1.0]);

/// assert!(!aabb2::intersects(&a, &b));
/// assert!(aabb2::intersects(&a, &c));
/// assert!(aabb2::intersects(&b, &c));
/// ~~~
#[inline]
pub fn intersects<T>(a: &AABB2<T>, b: &AABB2<T>) -> bool
where
    T: PartialOrd,
{
    return &b.max[0] >= &a.min[0]
        && &b.min[0] <= &a.max[0]
        && &b.max[1] >= &a.min[1]
        && &b.min[1] <= &a.max[1];
}
/// # Examples
/// ~~~
/// use aabb2;
/// let a = aabb2::new([0.0, 0.0], [2.0, 2.0]);
/// let mut v = [0.0, 0.0];
/// assert_eq!(aabb2::center(&mut v, &a), &[1.0, 1.0]);
/// ~~~
#[inline]
pub fn center<'out, T>(out: &'out mut [T; 2], aabb: &AABB2<T>) -> &'out mut [T; 2]
where
    T: FromPrimitive,
    for<'a, 'b> &'a T: Div<&'b T, Output = T> + Add<&'b T, Output = T> + Div<&'b T, Output = T>,
{
    out[0] = &(&aabb.min[0] + &aabb.max[0]) / &T::from_usize(2).unwrap();
    out[1] = &(&aabb.min[1] + &aabb.max[1]) / &T::from_usize(2).unwrap();
    out
}
