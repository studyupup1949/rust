pub trait ReflectTuple {
    const COUNT: usize;
    fn count(&self) -> usize {
        Self::COUNT
    }
}

macro_rules! impl_reflect_tuple {
    () => {
        impl ReflectTuple for () {
            const COUNT: usize = 0;
        }
    };
    ($($T:ident),+) => {
        impl<$($T),+> ReflectTuple for ($($T,)+) {
            const COUNT: usize = count_tuple_elems!($($T),+);
        }
    };
}

macro_rules! count_tuple_elems {
    () => { 0 };
    ($head:ident) => { 1 };
    ($head:ident, $($tail:ident),+) => { 1 + count_tuple_elems!($($tail),+) };
}

impl_reflect_tuple!();
impl_reflect_tuple!(A);
impl_reflect_tuple!(A, B);
impl_reflect_tuple!(A, B, C);
impl_reflect_tuple!(A, B, C, D);
impl_reflect_tuple!(A, B, C, D, E);
impl_reflect_tuple!(A, B, C, D, E, F);
impl_reflect_tuple!(A, B, C, D, E, F, G);
impl_reflect_tuple!(A, B, C, D, E, F, G, H);
impl_reflect_tuple!(A, B, C, D, E, F, G, H, I);
impl_reflect_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_reflect_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_reflect_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_reflect_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_reflect_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_reflect_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_reflect_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
