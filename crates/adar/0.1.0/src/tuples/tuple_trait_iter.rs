use crate::tuples::AsTraitRef;

pub struct TupleTraitIter<'a, T, const N: usize>
where
    T: ?Sized,
{
    tuple: [&'a T; N],
    index: usize,
}

impl<'a, T, const N: usize> Iterator for TupleTraitIter<'a, T, N>
where
    T: ?Sized,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < N {
            let item = &self.tuple[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
pub trait TupleIteratorTrait<T, const N: usize>
where
    T: ?Sized,
{
    fn iter(&self) -> TupleTraitIter<'_, T, N>;
}

impl<T> TupleIteratorTrait<T, 0> for ()
where
    T: ?Sized,
{
    fn iter(&self) -> TupleTraitIter<'_, T, 0> {
        TupleTraitIter {
            tuple: [],
            index: 0,
        }
    }
}

macro_rules! impl_tuple_trait {
    ($n:literal, ($($idx:tt => $T:ident),*)) => {
        #[allow(unused_parens)]
        impl<T, $($T),*> TupleIteratorTrait<T, $n> for ($($T),*,)
        where
            $($T: AsTraitRef<T>),*,
            T: ?Sized,
        {
            fn iter(&self) -> TupleTraitIter<'_, T, $n> {
                TupleTraitIter {
                    tuple: [ $( &self.$idx.as_trait_ref() ),* ],
                    index: 0,
                }
            }
        }
    };
}
impl_tuple_trait!(1, (0 => A));
impl_tuple_trait!(2, (0 => A, 1 => B));
impl_tuple_trait!(3, (0 => A, 1 => B, 2 => C));
impl_tuple_trait!(4, (0 => A, 1 => B, 2 => C, 3 => D));
impl_tuple_trait!(5, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E));
impl_tuple_trait!(6, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F));
impl_tuple_trait!(7, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G));
impl_tuple_trait!(8, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H));
impl_tuple_trait!(9, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I));
impl_tuple_trait!(10, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J));
impl_tuple_trait!(11, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K));
impl_tuple_trait!(12, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L));
impl_tuple_trait!(13, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M));
impl_tuple_trait!(14, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N));
impl_tuple_trait!(15, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O));
impl_tuple_trait!(16, (0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P));
