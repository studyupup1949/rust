use number_traits::Num;


#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AABB2<T: Copy + Num> {
    pub min: [T; 2],
    pub max: [T; 2],
}
