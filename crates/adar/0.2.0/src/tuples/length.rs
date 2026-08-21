pub trait TupleLength {
    fn len(&self) -> usize;
}

impl TupleLength for () {
    fn len(&self) -> usize {
        0
    }
}

macro_rules! impl_tuple_length {
    ($N:tt, $($T:ident),+) => {
        impl<$($T),+> TupleLength for ($($T,)+) {
            fn len(&self) -> usize {
                $N
            }
        }
    };
}

impl_tuple_length!(1, A);
impl_tuple_length!(2, A, B);
impl_tuple_length!(3, A, B, C);
impl_tuple_length!(4, A, B, C, D);
impl_tuple_length!(5, A, B, C, D, E);
impl_tuple_length!(6, A, B, C, D, E, F);
impl_tuple_length!(7, A, B, C, D, E, F, G);
impl_tuple_length!(8, A, B, C, D, E, F, G, H);
impl_tuple_length!(9, A, B, C, D, E, F, G, H, I);
impl_tuple_length!(10, A, B, C, D, E, F, G, H, I, J);
impl_tuple_length!(11, A, B, C, D, E, F, G, H, I, J, K);
impl_tuple_length!(12, A, B, C, D, E, F, G, H, I, J, K, L);
impl_tuple_length!(13, A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_tuple_length!(14, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_tuple_length!(15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_tuple_length!(16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tuple_length() {
        assert_eq!(().len(), 0);
        assert_eq!((0,).len(), 1);
        assert_eq!((0, 1).len(), 2);
        assert_eq!(
            (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15).len(),
            16
        );
    }
}
