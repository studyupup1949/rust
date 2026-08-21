use num_traits::Bounded;

use super::new_identity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AABB3<T> {
    pub min: [T; 3],
    pub max: [T; 3],
}

unsafe impl<T: Send> Send for AABB3<T> {}
unsafe impl<T: Sync> Sync for AABB3<T> {}

impl<T> Default for AABB3<T>
where
    T: Clone + Bounded,
{
    #[inline(always)]
    fn default() -> Self {
        new_identity()
    }
}

impl<T> AABB3<T>
where
    T: Clone + Bounded,
{
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }
}
