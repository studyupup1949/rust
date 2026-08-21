use number_traits::Num;

use super::new_identity;


#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AABB2<T: Copy + Num> {
    pub min: [T; 2],
    pub max: [T; 2],
}

impl<T: Copy + Num> Default for AABB2<T> {
    #[inline(always)]
    fn default() -> Self {
        new_identity()
    }
}
