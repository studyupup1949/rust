use number_traits::Num;
use vec3;

use super::AABB3;


/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// assert_eq!(&aabb3::new([0, 0, 0], [1, 1, 1]), &AABB3{ min: [0, 0, 0], max: [1, 1, 1] });
/// ~~~
#[inline(always)]
pub fn new<T>(min: [T; 3], max: [T; 3]) -> AABB3<T>
    where T: Copy + Num,
{
    AABB3 {
        min: min,
        max: max,
    }
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// assert_eq!(&aabb3::new_identity::<u8>(), &AABB3{ min: [255, 255, 255], max: [0, 0, 0] });
/// ~~~
#[inline(always)]
pub fn new_identity<T>() -> AABB3<T>
    where T: Copy + Num,
{
    let min = T::min_value();
    let max = T::max_value();
    AABB3 {
        min: [max; 3],
        max: [min; 3],
    }
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// assert_eq!(aabb3::set(&mut a, &[0, 0, 0], &[1, 1, 1]), &AABB3{ min: [0, 0, 0], max: [1, 1, 1] });
/// ~~~
#[inline]
pub fn set<'out, 'a, 'b, T>(out: &'out mut AABB3<T>, min: &'a [T; 3], max: &'b [T; 3]) -> &'out mut AABB3<T>
    where T: Copy + Num,
{
    vec3::copy(&mut out.min, min);
    vec3::copy(&mut out.max, max);
    out
}
/// # Examples
/// ~~~
/// use aabb3::{self, AABB3};
/// let mut a = aabb3::new_identity();
/// let b = aabb3::new([0, 0, 0], [1, 1, 1]);
/// assert_eq!(aabb3::copy(&mut a, &b), &AABB3{ min: [0, 0, 0], max: [1, 1, 1] });
/// ~~~
#[inline]
pub fn copy<'out, 'a, T>(out: &'out mut AABB3<T>, a: &'a AABB3<T>) -> &'out mut AABB3<T>
    where T: Copy + Num,
{
    vec3::copy(&mut out.min, &a.min);
    vec3::copy(&mut out.max, &a.max);
    out
}
