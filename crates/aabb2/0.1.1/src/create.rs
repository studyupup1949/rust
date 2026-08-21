use number_traits::Num;
use vec2;

use super::AABB2;


/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// assert_eq!(&aabb2::new([0, 0], [1, 1]), &AABB2{ min: [0, 0], max: [1, 1] });
/// ~~~
#[inline(always)]
pub fn new<T>(min: [T; 2], max: [T; 2]) -> AABB2<T>
    where T: Copy + Num,
{
    AABB2 {
        min: min,
        max: max,
    }
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// assert_eq!(&aabb2::new_identity::<u8>(), &AABB2{ min: [255, 255], max: [0, 0] });
/// ~~~
#[inline(always)]
pub fn new_identity<T>() -> AABB2<T>
    where T: Copy + Num,
{
    let min = T::min_value();
    let max = T::max_value();
    AABB2 {
        min: [max; 2],
        max: [min; 2],
    }
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// assert_eq!(aabb2::set(&mut a, &[0, 0], &[1, 1]), &AABB2{ min: [0, 0], max: [1, 1] });
/// ~~~
#[inline]
pub fn set<'out, 'a, 'b, T>(out: &'out mut AABB2<T>, min: &'a [T; 2], max: &'b [T; 2]) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    vec2::copy(&mut out.min, min);
    vec2::copy(&mut out.max, max);
    out
}
/// # Examples
/// ~~~
/// use aabb2::{self, AABB2};
/// let mut a = aabb2::new_identity();
/// let b = aabb2::new([0, 0], [1, 1]);
/// assert_eq!(aabb2::copy(&mut a, &b), &AABB2{ min: [0, 0], max: [1, 1] });
/// ~~~
#[inline]
pub fn copy<'out, 'a, T>(out: &'out mut AABB2<T>, a: &'a AABB2<T>) -> &'out mut AABB2<T>
    where T: Copy + Num,
{
    vec2::copy(&mut out.min, &a.min);
    vec2::copy(&mut out.max, &a.max);
    out
}
