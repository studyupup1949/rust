use number_traits::Num;

use super::new_identity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AABB3<T: Copy + Num> {
    pub min: [T; 3],
    pub max: [T; 3],
}

unsafe impl<T: Send + Copy + Num> Send for AABB3<T> {}
unsafe impl<T: Sync + Copy + Num> Sync for AABB3<T> {}

impl<T: Copy + Num> Default for AABB3<T> {
    #[inline(always)]
    fn default() -> Self {
        new_identity()
    }
}
