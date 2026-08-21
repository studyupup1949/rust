use num_traits::Bounded;

use super::new_identity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AABB2<T> {
    pub min: [T; 2],
    pub max: [T; 2],
}

unsafe impl<T: Send> Send for AABB2<T> {}
unsafe impl<T: Sync> Sync for AABB2<T> {}

impl<T> Default for AABB2<T>
where
    T: Clone + Bounded,
{
    #[inline(always)]
    fn default() -> Self {
        new_identity()
    }
}

impl<T> AABB2<T>
where
    T: Clone + Bounded,
{
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }
}
